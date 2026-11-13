use super::{EvaluationContext, EvaluationState, evaluation_step::EvaluationStep, instruction::Instruction, operation, value_type};
use materialcraft_core::Value;

/// Validated graph with input ids resolved to instruction indices
pub struct Program {
    pub(crate) instructions: Vec<Instruction>,
    pub(crate) output: usize,
    pub(crate) seed: u32,
}

const MAX_EVALUATION_STEPS: usize = 100_000;

impl Program {
    pub fn output_type(&self) -> u32 {
        self.instructions.get(self.output).map_or(0, |instruction| instruction.type_code)
    }

    pub fn node_type(&self, id: u32) -> Option<u32> {
        self.instructions.iter().find(|instruction| instruction.id == id).map(|instruction| instruction.type_code)
    }

    fn instruction(&self, index: usize) -> Result<&Instruction, String> {
        self.instructions.get(index).ok_or_else(|| "invalid function instruction".into())
    }

    fn input(&self, index: usize, slot: usize) -> Result<usize, String> {
        self.instruction(index)?.inputs.get(slot).copied().ok_or_else(|| "missing function input".into())
    }

    pub fn evaluate(&self, context: &EvaluationContext<'_>, state: &mut EvaluationState) -> Result<Value, String> {
        let mut random = self.random_seed(context);
        self.evaluate_with_random(context, state, &mut random)
    }

    /// Pattern parameters share a random stream within each traversal visit.
    pub(crate) fn evaluate_with_random(&self, context: &EvaluationContext<'_>, state: &mut EvaluationState, random: &mut u32) -> Result<Value, String> {
        state.variables.clone_from(context.variables);
        state.random = *random;
        let mut remaining = MAX_EVALUATION_STEPS;
        let result = self.run(context, state, &mut remaining);
        *random = state.random;
        let value = result?;
        // Nested calls can return undefined intermediates to a caller that clamps
        // them, but a completed parameter/value function must publish finite data.
        if value.as_vec4().is_some_and(|components| components.iter().any(|value| !value.is_finite())) {
            return Err("function produced a non-finite output".into());
        }
        Ok(value)
    }

    fn random_seed(&self, context: &EvaluationContext<'_>) -> u32 {
        let seed = context.variables.get("$randomseed").and_then(Value::as_i32).map_or(self.seed, |seed| seed as u32);
        if context.scope == super::FunctionScope::PixelProcessor {
            // Each pixel owns a deterministic stream, independent of worker scheduling.
            let x = (context.position[0] * context.size[0]).floor() as u32;
            let y = (context.position[1] * context.size[1]).floor() as u32;
            crate::random::hash(seed ^ crate::random::hash(x.wrapping_add(0x9e37_79b9)) ^ crate::random::hash(y.wrapping_add(0x85eb_ca6b)))
        } else {
            seed
        }
    }

    // Pixel assignments are local; only the general evaluator exports the complete environment
    pub(super) fn evaluate_pixel(&self, context: &EvaluationContext<'_>, state: &mut EvaluationState) -> Result<Value, String> {
        state.variables.clear();
        state.random = self.random_seed(context);
        let mut remaining = MAX_EVALUATION_STEPS;
        self.run(context, state, &mut remaining)
    }

    fn run(&self, context: &EvaluationContext<'_>, state: &mut EvaluationState, remaining_steps: &mut usize) -> Result<Value, String> {
        use EvaluationStep::*;

        state.steps.clear();
        state.values.clear();
        state.assignments.clear();
        state.registers.clear();
        state.registers.resize(self.instructions.len(), None);
        state.steps.push(Enter(self.output));

        while let Some(step) = state.steps.pop() {
            *remaining_steps = remaining_steps.checked_sub(1).ok_or("function exceeded its execution budget; check loop termination")?;
            match step {
                Enter(index) => {
                    let instruction = self.instruction(index)?;
                    if let Some(Some(value)) = state.registers.get(index) {
                        state.values.push(value.clone());
                        continue;
                    }

                    state.steps.push(Store(index));

                    match instruction.definition.id {
                        "ifelse" => {
                            state.steps.push(Branch(index));
                            state.steps.push(Enter(self.input(index, 0)?));
                        }
                        "sequence" => {
                            state.steps.push(Sequence(index));
                            state.steps.push(Enter(self.input(index, 0)?));
                        }
                        "while" => {
                            state.steps.push(LoopStart(index));
                            state.steps.push(Enter(self.input(index, 0)?));
                        }
                        _ => {
                            state.steps.push(Apply { instruction: index, base: state.values.len() });
                            state.steps.extend(instruction.inputs.iter().rev().map(|index| Enter(*index)));
                        }
                    }
                }
                Apply { instruction: index, base } => {
                    let instruction = self.instruction(index)?;
                    let arguments = state.values.get(base..).ok_or("invalid function operands")?;
                    let result = if let Some(call) = &instruction.call {
                        let storage = state.calls.entry(index).or_default();
                        storage.variables.clone_from(&state.variables);
                        storage.variables.extend(call.defaults.clone());
                        for (name, value) in call.inputs.iter().zip(arguments) {
                            storage.variables.insert(name.clone(), value.clone());
                        }
                        // A function call consumes its caller's stream instead of restarting it.
                        storage.random = state.random;
                        let result = call.program.run(context, storage, remaining_steps);
                        state.random = storage.random;
                        if call.exports && result.is_ok() {
                            // Set publishes into the caller's scope; arguments and defaults stay local.
                            state.variables.extend(storage.assignments.clone());
                            state.assignments.extend(storage.assignments.clone());
                        }
                        result
                    } else {
                        operation::evaluate(instruction, arguments, &mut state.variables, &mut state.random, context)
                    }
                    .map_err(|error| format!("{} ({}): {error}", instruction.definition.label, instruction.id))?;

                    if instruction.definition.id == "set"
                        && let Some(Value::Text(name)) = &instruction.value
                    {
                        state.assignments.insert(name.clone(), result.clone());
                    }

                    state.values.truncate(base);
                    state.values.push(result);
                }
                Store(index) => {
                    let value = state.values.last().ok_or("missing function result")?;
                    let register = state.registers.get_mut(index).ok_or("invalid function register")?;
                    *register = Some(value.clone());
                }
                Branch(index) => {
                    let condition = state.values.pop().ok_or("missing branch condition")?;
                    let slot = if value_type::boolean(&condition)? { 1 } else { 2 };

                    state.steps.push(Enter(self.input(index, slot)?));
                }
                Sequence(index) => {
                    state.values.pop().ok_or("missing sequence result")?;
                    state.steps.push(Enter(self.input(index, 1)?));
                }
                LoopStart(index) => {
                    let last = state.values.pop().ok_or("missing loop initialization")?;
                    self.clear_loop_results(index, state)?;
                    state.steps.push(LoopCondition { instruction: index, iteration: 0, last });
                    state.steps.push(Enter(self.input(index, 1)?));
                }
                LoopCondition { instruction: index, iteration, last } => {
                    let exit = value_type::boolean(&state.values.pop().ok_or("missing loop condition")?)?;
                    let maximum = self.instruction(index)?.value.as_ref().and_then(Value::as_i32).unwrap_or(100);
                    if exit || (maximum >= 0 && iteration >= maximum as u32) {
                        state.values.push(last);
                    } else {
                        self.clear_loop_results(index, state)?;
                        state.steps.push(LoopBody { instruction: index, iteration: iteration.saturating_add(1) });
                        state.steps.push(Enter(self.input(index, 2)?));
                    }
                }
                LoopBody { instruction: index, iteration } => {
                    let last = state.values.pop().ok_or("missing loop body result")?;
                    self.clear_loop_results(index, state)?;
                    state.steps.push(LoopCondition { instruction: index, iteration, last });
                    state.steps.push(Enter(self.input(index, 1)?));
                }
            }
        }

        let result = state.values.pop().ok_or("function returned no value")?;
        let expected = self.instruction(self.output)?.type_code;
        if value_type::type_code(&result) != expected {
            return Err(format!("function returned {}, expected native type {expected}", result.type_name()));
        }

        Ok(result)
    }

    fn clear_loop_results(&self, index: usize, state: &mut EvaluationState) -> Result<(), String> {
        // A node's outgoing wires share its computed value, including reads, branches and side effects.
        // Only the current loop's condition and body run again; unrelated cached values stay intact.
        for repeated in &self.instruction(index)?.repeated {
            if let Some(register) = state.registers.get_mut(*repeated) {
                *register = None;
            }
        }
        Ok(())
    }
}

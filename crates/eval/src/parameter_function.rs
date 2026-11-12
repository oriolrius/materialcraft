use materialcraft_core::{FunctionGraph, Value, ValueType};
use materialcraft_nodes::function::{EvaluationContext, EvaluationState, compile};
use std::collections::BTreeMap;

pub(crate) fn evaluate(
    function: &FunctionGraph,
    expected: ValueType,
    variables: &mut BTreeMap<String, Value>,
    seed: u32,
    size: [u32; 2],
    allow_scalar: bool,
) -> Result<Value, String> {
    let program = compile(function, seed)?;
    let scalar_channel = allow_scalar && expected == ValueType::Float4 && program.output_type() == ValueType::Float as u32;
    if program.output_type() != expected as u32 && !scalar_channel {
        return Err(format!("output must be {}", expected.label()));
    }

    let context = EvaluationContext { scope: Default::default(), position: [0.5; 2], size: size.map(|side| side as f32), images: &[], tile: true, variables };
    let mut state = EvaluationState::default();
    let value = program.evaluate(&context, &mut state)?;
    let finite = match &value {
        Value::Float(value) => value.is_finite(),
        Value::Float2(values) => values.iter().all(|value| value.is_finite()),
        Value::Float3(values) => values.iter().all(|value| value.is_finite()),
        Value::Float4(values) | Value::Color(values) => values.iter().all(|value| value.is_finite()),
        _ => true,
    };
    if !finite {
        return Err("function returned a non-finite value".into());
    }

    variables.extend(state.assignments().clone());

    Ok(value)
}

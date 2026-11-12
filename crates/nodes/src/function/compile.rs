use super::{Program, catalog, instruction::Instruction};
use materialcraft_core::FunctionGraph;
use std::collections::{BTreeMap, BTreeSet};

pub fn compile(graph: &FunctionGraph, seed: u32) -> Result<Program, String> {
    let mut remaining = super::MAX_EXPANDED_NODES;
    compile_nested(graph, seed, 0, &mut remaining)
}

fn compile_nested(graph: &FunctionGraph, seed: u32, depth: usize, remaining: &mut usize) -> Result<Program, String> {
    if graph.nodes.len() > 4096 {
        return Err("function exceeds the 4096-node graph limit".into());
    }
    if depth > super::MAX_CALL_DEPTH || graph.nodes.len() > *remaining {
        return Err("function exceeds the 16-level or 65536-node expanded execution limit".into());
    }
    *remaining -= graph.nodes.len();
    let output = super::output_node(graph).ok_or("function has no result node")?.id;
    let mut nodes = BTreeMap::new();
    for node in &graph.nodes {
        if nodes.insert(node.id, node).is_some() {
            return Err(format!("duplicate function node {}", node.id));
        }
    }

    let mut pending = vec![(output, false)];
    let mut active = BTreeSet::new();
    let mut indices = BTreeMap::new();
    let mut instructions: Vec<Instruction> = Vec::new();
    while let Some((id, visited)) = pending.pop() {
        if indices.contains_key(&id) {
            continue;
        }

        let node = nodes.get(&id).ok_or_else(|| format!("missing function node {id}"))?;
        if node.missing.is_some() {
            return Err(format!("Missing function operation `{}`", node.op));
        }
        let definition = if node.instance.is_some() {
            &super::function_call::DEFINITION
        } else {
            catalog::definition(&node.op).ok_or_else(|| format!("unknown function `{}`", node.op))?
        };
        let names: Vec<_> = super::input_identifiers(node).collect();
        // An operation computes with zero on an input nothing is connected to; a call needs a source default instead
        let missing_input = node
            .instance
            .as_ref()
            .is_some_and(|instance| names.iter().zip(&node.inputs).any(|(name, input)| input.is_none() && !instance.defaults.contains_key(*name)));
        if node.inputs.len() != names.len() || missing_input {
            return Err(format!("{} ({id}) requires connected inputs: {}", super::node_label(node), names.join(", ")));
        }

        // Preserved legacy connectors must not evaluate a duplicate random or stateful input twice
        let ports: Vec<_> = names
            .into_iter()
            .zip(&node.inputs)
            .filter(|(name, _)| node.instance.as_ref().is_none_or(|instance| instance.defaults.contains_key(*name)))
            .collect();
        let sources: Vec<_> = ports.iter().filter_map(|(_, input)| **input).collect();
        if !visited {
            if !active.insert(id) {
                return Err(format!("function cycle at node {id}"));
            }

            pending.push((id, true));
            pending.extend(sources.iter().rev().map(|input| (*input, false)));
            continue;
        }

        active.remove(&id);
        let connected: Vec<usize> =
            sources.iter().map(|input| indices.get(input).copied().ok_or("unresolved function input".to_string())).collect::<Result<_, _>>()?;
        let inputs = if node.instance.is_some() || ports.iter().all(|(_, input)| input.is_some()) {
            connected
        } else {
            let sibling = connected.first().and_then(|index| instructions.get(*index)).map(|instruction| instruction.type_code);
            let mut connected = connected.into_iter();
            let mut inputs = Vec::with_capacity(ports.len());
            for (name, input) in &ports {
                let index = match input {
                    Some(_) => connected.next().ok_or("unresolved function input")?,
                    None => {
                        let (definition, value) = unconnected(name, sibling.unwrap_or(node.type_code))?;
                        let type_code = super::value_type::type_code(&value);
                        instructions.push(Instruction {
                            id: 0,
                            definition,
                            inputs: Vec::new(),
                            value: Some(value),
                            type_code,
                            repeated: Vec::new(),
                            call: None,
                        });
                        instructions.len() - 1
                    }
                };
                inputs.push(index);
            }
            inputs
        };
        let call = if let Some(instance) = &node.instance {
            let source = node.source.as_ref().ok_or_else(|| format!("unresolved Function Graph `{}`", instance.graph))?.as_ref().map_err(Clone::clone)?;
            let program = compile_nested(source, seed, depth + 1, remaining)?;
            for (port, index) in instance
                .inputs
                .iter()
                .zip(&node.inputs)
                .filter(|(port, input)| input.is_some() && instance.defaults.contains_key(&port.identifier))
                .map(|(port, _)| port)
                .zip(&inputs)
            {
                if instructions.get(*index).is_some_and(|instruction| instruction.type_code != port.type_code) {
                    return Err(format!("{}: input `{}` has the wrong type", instance.graph, port.identifier));
                }
            }
            let inputs = ports.iter().filter(|(_, input)| input.is_some()).map(|(name, _)| (*name).to_string()).collect();
            let exports =
                program.instructions.iter().any(|instruction| instruction.definition.id == "set" || instruction.call.as_ref().is_some_and(|call| call.exports));
            Some(super::function_call::FunctionCall { program, inputs, defaults: instance.defaults.clone(), exports })
        } else {
            None
        };
        let value = node.value.clone().or_else(|| catalog::default_value(&node.op));
        let input_type = |index: usize| inputs.get(index).and_then(|index| instructions.get(*index)).map_or(256, |instruction| instruction.type_code);
        let inferred = super::inferred_type::inferred_type(&node.op, value.as_ref(), input_type);
        let type_code = call.as_ref().map_or_else(|| if node.type_code == 0 { inferred } else { node.type_code }, |call| call.program.output_type());
        indices.insert(id, instructions.len());
        instructions.push(Instruction { id, definition, inputs, value, type_code, repeated: Vec::new(), call });
    }

    // Instructions come after their inputs, so walking back from a loop reaches its whole condition and body
    for index in 0..instructions.len() {
        if instructions.get(index).is_some_and(|instruction| instruction.definition.id == "while") {
            let repeated = loop_members(&instructions, index);
            if let Some(instruction) = instructions.get_mut(index) {
                instruction.repeated = repeated;
            }
        }
    }

    let output = indices.get(&output).copied().ok_or("missing result instruction")?;
    Ok(Program { instructions, output, seed })
}

/// The instructions the condition and the body of loop `index` evaluate again on every iteration
fn loop_members(instructions: &[Instruction], index: usize) -> Vec<usize> {
    let mut member = vec![false; instructions.len()];
    let mut pending: Vec<usize> = instructions.get(index).map(|instruction| instruction.inputs.iter().skip(1).copied().collect()).unwrap_or_default();
    while let Some(next) = pending.pop() {
        if let Some(seen) = member.get_mut(next)
            && !*seen
        {
            *seen = true;
            pending.extend(instructions.get(next).map(|instruction| instruction.inputs.as_slice()).unwrap_or_default());
        }
    }

    member.iter().enumerate().filter(|(_, member)| **member).map(|(index, _)| index).collect()
}

/// The constant an unconnected input `name` reads: false for a condition, otherwise zero of the type its node works with
fn unconnected(name: &str, type_code: u32) -> Result<(&'static super::FunctionDefinition, materialcraft_core::Value), String> {
    use materialcraft_core::ValueType;

    let value_type = if matches!(name, "cond" | "condition") { ValueType::Boolean } else { ValueType::from_code(type_code).unwrap_or(ValueType::Float) };
    let operation = match value_type {
        ValueType::Boolean => "const_bool",
        ValueType::Integer => "const_int1",
        ValueType::Integer2 => "const_int2",
        ValueType::Integer3 => "const_int3",
        ValueType::Integer4 => "const_int4",
        ValueType::Float => "const_float1",
        ValueType::Float2 => "const_float2",
        ValueType::Float3 => "const_float3",
        ValueType::Float4 => "const_float4",
        ValueType::String => "const_string",
    };
    let definition = catalog::definition(operation).ok_or("missing constant operation")?;
    Ok((definition, value_type.default_value()))
}

use materialcraft_core::{FunctionGraph, Value, ValueType};
use materialcraft_nodes::function::{EvaluationContext, EvaluationState, compile};
use std::collections::BTreeMap;

pub(crate) fn evaluate(
    function: &FunctionGraph,
    expected: ValueType,
    variables: &BTreeMap<String, Value>,
    seed: u32,
    size: [u32; 2],
    allow_scalar: bool,
) -> Result<Value, String> {
    evaluate_keeping_variables(function, expected, &mut variables.clone(), seed, size, allow_scalar)
}

/// Like `evaluate`, and the user variables the function set stay in `variables` for the functions evaluated after it
pub(crate) fn evaluate_keeping_variables(
    function: &FunctionGraph,
    expected: ValueType,
    variables: &mut BTreeMap<String, Value>,
    seed: u32,
    size: [u32; 2],
    allow_scalar: bool,
) -> Result<Value, String> {
    let program = compile(function, seed)?;
    let scalar_channel = allow_scalar && expected == ValueType::Float4 && program.output_type() == ValueType::Float as u32;
    // A color parameter also takes a gray level or an opaque RGB triple
    let widened = expected == ValueType::Float4 && [ValueType::Float as u32, ValueType::Float3 as u32].contains(&program.output_type());
    if program.output_type() != expected as u32 && !scalar_channel && !widened {
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

    let value = match value {
        Value::Float(level) if !scalar_channel && widened => Value::Float4([level, level, level, 1.0]),
        Value::Float3([red, green, blue]) if widened => Value::Float4([red, green, blue, 1.0]),
        value => value,
    };

    for (name, set) in state.variables() {
        if !name.starts_with('$') && variables.get(name) != Some(set) {
            variables.insert(name.clone(), set.clone());
        }
    }

    Ok(value)
}

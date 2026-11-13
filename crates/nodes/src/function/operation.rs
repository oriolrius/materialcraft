use super::{
    EvaluationContext,
    instruction::Instruction,
    value_type::{self, boolean, dimensions, floats, integers},
};
use materialcraft_core::Value;
use std::collections::BTreeMap;

pub(crate) fn evaluate(
    instruction: &Instruction,
    arguments: &[Value],
    variables: &mut BTreeMap<String, Value>,
    random: &mut u32,
    context: &EvaluationContext<'_>,
) -> Result<Value, String> {
    let op = instruction.definition.id;
    let first = || arguments.first().ok_or("missing first operand");
    let second = || arguments.get(1).ok_or("missing second operand");
    let name = || match &instruction.value {
        Some(Value::Text(name)) => Ok(name.as_str()),
        _ => Err("variable name must be a string".to_string()),
    };

    if op.starts_with("const_") {
        let value = instruction.value.clone().ok_or("constant has no value")?;
        if value_type::type_code(&value) != instruction.type_code {
            return Err("constant has the wrong type".into());
        }

        return value_type::canonical(value);
    }

    if op.starts_with("get_") {
        let name = name()?;
        let missing = || format!("undefined variable `{name}` in this context");
        let value = match context.variable(name, variables)? {
            Some(value) => value,
            // User counters can be read before their first Set. They start at
            // typed zero; unavailable system variables must still report errors.
            None if !name.starts_with('$') => materialcraft_core::ValueType::from_code(instruction.type_code)
                .filter(|kind| *kind != materialcraft_core::ValueType::String)
                .map(materialcraft_core::ValueType::default_value)
                .ok_or_else(missing)?,
            None => return Err(missing()),
        };

        // Older authored graphs read an exposed integer option as a Boolean.
        // Keep system-variable type checks strict; user options use zero/nonzero.
        if op == "get_bool"
            && !name.starts_with('$')
            && matches!(value, Value::Int(_) | Value::Enum(_))
            && let Some(integer) = value.as_i32()
        {
            return Ok(Value::Bool(integer != 0));
        }

        if value_type::type_code(&value) != instruction.type_code {
            return Err(format!("variable `{name}` has the wrong type for {op}"));
        }

        return value_type::canonical(value);
    }

    if op.starts_with("to") || op.contains("vector") || op.contains("swizzle") {
        return super::vector::evaluate(op, arguments, instruction.value.as_ref());
    }

    Ok(match op {
        "passthrough" => first()?.clone(),
        "set" => {
            let name = name()?;
            if name.is_empty() || name.starts_with('$') {
                return Err("use a nonempty user variable name".into());
            }

            let value = first()?.clone();
            variables.insert(name.to_string(), value.clone());
            value
        }
        "isdefined" => {
            let name = name()?;
            Value::Bool(context.variable(name, variables)?.is_some())
        }
        "and" => Value::Bool(boolean(first()?)? && boolean(second()?)?),
        "or" => Value::Bool(boolean(first()?)? || boolean(second()?)?),
        "not" => Value::Bool(!boolean(first()?)?),
        "eq" | "noteq" => {
            let (left, right) = (first()?, second()?);
            if value_type::type_code(left) != value_type::type_code(right) {
                return Err("comparison inputs must have the same type".into());
            }
            Value::Bool((left == right) == (op == "eq"))
        }
        "gt" | "gteq" | "lr" | "lreq" => {
            let (left, right) = (first()?, second()?);
            if dimensions(left) != 1 || value_type::type_code(left) != value_type::type_code(right) {
                return Err("comparison requires matching scalars".into());
            }
            let ordering = match (left, right) {
                (Value::Int(a), Value::Int(b)) => a.partial_cmp(b),
                (Value::Float(a), Value::Float(b)) => a.partial_cmp(b),
                (Value::Bool(a), Value::Bool(b)) => a.partial_cmp(b),
                _ => return Err("comparison requires numeric inputs".into()),
            };
            // Ordered IEEE comparisons are false for NaN, including <= and >=.
            // Undefined intermediates can therefore select a finite fallback.
            Value::Bool(ordering.is_some_and(|ordering| match op {
                "gt" => ordering.is_gt(),
                "gteq" => ordering.is_ge(),
                "lr" => ordering.is_lt(),
                _ => ordering.is_le(),
            }))
        }
        "samplecol" | "samplelum" => {
            let position = floats(first()?)?;
            if dimensions(first()?) != 2 {
                return Err("sample position must be float2".into());
            }
            let setting = instruction.value.as_ref().map(integers).transpose()?.unwrap_or([0, 0, 0, 0]);
            let index = usize::try_from(setting[0]).map_err(|_| "negative image index")?;
            let pixel = match context.images.get(index).and_then(Option::as_ref) {
                Some(image) if setting[1] == 0 => {
                    image.get((position[0] * image.w as f32).floor() as i64, (position[1] * image.h as f32).floor() as i64, context.tile)
                }
                Some(image) => image.sample_stored(position[0], position[1], context.tile),
                None => [0.0; 4],
            };

            if op == "samplelum" { Value::Float(pixel[0]) } else { Value::Float4(pixel) }
        }
        _ => return super::arithmetic::evaluate(op, arguments, random, context.scope),
    })
}

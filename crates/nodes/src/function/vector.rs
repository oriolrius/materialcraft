use super::value_type::{dimensions, floats, integer_value, integers, intermediate_float_value as float_value};
use materialcraft_core::Value;

pub(crate) fn evaluate(op: &str, args: &[Value], setting: Option<&Value>) -> Result<Value, String> {
    let first = args.first().ok_or("missing vector input")?;
    let count = op.chars().last().and_then(|c| c.to_digit(10)).unwrap_or(1) as usize;
    if !(1..=4).contains(&count) {
        return Err("invalid vector size".into());
    }
    let integer = op.starts_with('i') || op.starts_with("toint");
    let mut result_float = [0.0; 4];
    let mut result_int = [0; 4];
    if op.contains("swizzle") {
        let indices = setting.map(integers).transpose()?.unwrap_or([0; 4]);
        let floats = if integer { [0.0; 4] } else { floats(first)? };
        let integers = if integer { integers(first)? } else { [0; 4] };
        for (lane, index) in indices.iter().take(count).enumerate() {
            let index = usize::try_from(*index).ok().filter(|index| *index < 4).ok_or("swizzle component is outside its input")?;
            if integer {
                if let Some(result) = result_int.get_mut(lane) {
                    // Swizzles read zero in undeclared lanes, including scalar
                    // inputs. Arithmetic's scalar broadcast does not apply here.
                    *result = if index < dimensions(first) { *integers.get(index).ok_or("invalid swizzle")? } else { 0 };
                }
            } else if let Some(result) = result_float.get_mut(lane) {
                *result = if index < dimensions(first) { *floats.get(index).ok_or("invalid swizzle")? } else { 0.0 };
            }
        }
    } else if op.contains("vector") {
        let last = args.get(1).ok_or("missing final vector component")?;
        if dimensions(first) + dimensions(last) != count {
            return Err(format!("vector{count} requires inputs totaling {count} components"));
        }

        if integer {
            let left = integers(first)?;
            let right = integers(last)?;
            for (result, value) in result_int.iter_mut().zip(left.into_iter().take(dimensions(first)).chain(right.into_iter().take(dimensions(last)))) {
                *result = value;
            }
        } else {
            let left = floats(first)?;
            let right = floats(last)?;
            for (result, value) in result_float.iter_mut().zip(left.into_iter().take(dimensions(first)).chain(right.into_iter().take(dimensions(last)))) {
                *result = value;
            }
        }
    } else {
        if dimensions(first) != count {
            return Err("casts preserve the number of components".into());
        }
        if integer {
            let values = floats(first)?;
            for (result, value) in result_int.iter_mut().zip(values).take(count) {
                if !value.is_finite() || !(-2147483648.0..2147483648.0).contains(&value) {
                    return Err("float is outside the integer range".into());
                }
                // Integer conversion rounds down; truncating toward zero shifts
                // negative texture coordinates and lattice indices by one cell.
                *result = value.floor() as i32;
            }
        } else {
            for (result, value) in result_float.iter_mut().zip(integers(first)?).take(count) {
                *result = value as f32;
            }
        }
    }

    if integer {
        integer_value(result_int.get(..count).ok_or("invalid integer dimensions")?)
    } else {
        float_value(result_float.get(..count).ok_or("invalid float dimensions")?)
    }
}

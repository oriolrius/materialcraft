use materialcraft_engine::core::Value;

/// `value` as a node face shows it: short numbers, vector components separated by commas
pub(crate) fn value_text(value: &Value) -> String {
    match value {
        Value::Float(value) => number(*value),
        Value::Float2(values) => numbers(values),
        Value::Float3(values) => numbers(values),
        Value::Float4(values) | Value::Color(values) => numbers(values),
        Value::Int(value) => value.to_string(),
        Value::Int2(values) => integers(values),
        Value::Int3(values) => integers(values),
        Value::Int4(values) => integers(values),
        Value::Bool(value) => crate::i18n::tr(if *value { "True" } else { "False" }).into(),
        Value::Enum(value) => value.to_string(),
        Value::Text(value) => value.clone(),
        Value::Curve(_) | Value::Gradient(_) | Value::Function(_) | Value::PatternGraph(_) => String::new(),
    }
}

/// At most three decimals, without trailing zeros
fn number(value: f32) -> String {
    if !value.is_finite() {
        return value.to_string();
    }

    let text = format!("{value:.3}");
    let text = text.trim_end_matches('0').trim_end_matches('.');
    if text == "-0" { "0".into() } else { text.into() }
}

fn numbers(values: &[f32]) -> String {
    values.iter().map(|value| number(*value)).collect::<Vec<_>>().join(", ")
}

fn integers(values: &[i32]) -> String {
    values.iter().map(ToString::to_string).collect::<Vec<_>>().join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_are_short_and_vectors_list_their_components() {
        assert_eq!(value_text(&Value::Float(0.5)), "0.5");
        assert_eq!(value_text(&Value::Float(2.0)), "2");
        assert_eq!(value_text(&Value::Float(-0.0001)), "0");
        assert_eq!(value_text(&Value::Float(1.0 / 3.0)), "0.333");
        assert_eq!(value_text(&Value::Float(f32::NAN)), "NaN");
        assert_eq!(value_text(&Value::Float3([1.0, 0.25, -3.5])), "1, 0.25, -3.5");
        assert_eq!(value_text(&Value::Int2([4, -7])), "4, -7");
        assert_eq!(value_text(&Value::Bool(true)), "True");
        assert_eq!(value_text(&Value::Text("size".into())), "size");
    }
}

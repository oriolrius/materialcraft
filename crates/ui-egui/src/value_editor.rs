use egui::Ui;
use materialcraft_engine::core::Value;

/// Edit a function value within the available width
pub(crate) fn show(ui: &mut Ui, value: &mut Value) -> bool {
    ui.horizontal_wrapped(|ui| fields(ui, value, false)).inner
}

/// Edit a function value with one full-width field per component, as a node face shows it
pub(crate) fn show_stacked(ui: &mut Ui, value: &mut Value) -> bool {
    ui.vertical(|ui| fields(ui, value, true)).inner
}

/// How many fields `show_stacked` lays out for `value`
pub(crate) fn rows(value: &Value) -> usize {
    match value {
        Value::Float2(_) | Value::Int2(_) => 2,
        Value::Float3(_) | Value::Int3(_) => 3,
        Value::Float4(_) | Value::Int4(_) | Value::Color(_) => 4,
        _ => 1,
    }
}

fn fields(ui: &mut Ui, value: &mut Value, stacked: bool) -> bool {
    match value {
        Value::Float(value) => float(ui, value, "Value", stacked),
        Value::Float2(values) => floats(ui, values, &["X", "Y"], stacked),
        Value::Float3(values) => floats(ui, values, &["X", "Y", "Z"], stacked),
        Value::Float4(values) => floats(ui, values, &["X", "Y", "Z", "W"], stacked),
        Value::Color(values) => floats(ui, values, &["Red", "Green", "Blue", "Alpha"], stacked),
        Value::Int(value) => integer(ui, value, "Value", stacked),
        Value::Int2(values) => integers(ui, values, stacked),
        Value::Int3(values) => integers(ui, values, stacked),
        Value::Int4(values) => integers(ui, values, stacked),
        Value::Bool(value) => {
            let text = if !stacked {
                ""
            } else if *value {
                "True"
            } else {
                "False"
            };
            let response = ui.checkbox(value, crate::i18n::tr(text));
            crate::widgets::name(&response, egui::Role::CheckBox, "Value");
            response.changed()
        }
        Value::Text(value) => {
            let response = ui.add(egui::TextEdit::singleline(value).desired_width(ui.available_width()));
            crate::widgets::name(&response, egui::Role::TextInput, "Value");
            response.changed()
        }
        _ => false,
    }
}

/// A stacked field fills the row and names its component in front of the number
fn field(ui: &mut Ui, drag: egui::DragValue<'_>, label: &str, stacked: bool) -> bool {
    let response = if stacked {
        let drag = if label == "Value" { drag } else { drag.prefix(format!("{} ", label.chars().next().unwrap_or(' '))) };
        ui.add_sized([ui.available_width(), ui.spacing().interact_size.y], drag)
    } else {
        ui.add(drag)
    };
    crate::widgets::name(&response, egui::Role::SpinButton, label);
    response.changed()
}

fn float(ui: &mut Ui, value: &mut f32, label: &str, stacked: bool) -> bool {
    field(ui, egui::DragValue::new(value).speed(0.01).max_decimals(3), label, stacked)
}

fn floats(ui: &mut Ui, values: &mut [f32], labels: &[&str], stacked: bool) -> bool {
    let mut changed = false;
    for (value, label) in values.iter_mut().zip(labels) {
        changed |= float(ui, value, label, stacked);
    }
    changed
}

fn integer(ui: &mut Ui, value: &mut i32, label: &str, stacked: bool) -> bool {
    field(ui, egui::DragValue::new(value).speed(0.1), label, stacked)
}

fn integers(ui: &mut Ui, values: &mut [i32], stacked: bool) -> bool {
    let mut changed = false;
    for (value, label) in values.iter_mut().zip(["X", "Y", "Z", "W"]) {
        changed |= integer(ui, value, label, stacked);
    }
    changed
}

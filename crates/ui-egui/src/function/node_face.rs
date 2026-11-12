//! What a function node shows in its body: its editable value, or the symbol of its operation

use egui::{Rect, Ui, vec2};
use egui_phosphor::regular as icon;
use materialcraft_engine::core::{FunctionNode, Value};

/// Graph-space height of one value field, and the gap between two fields
const ROW_HEIGHT: f32 = 20.0;
const ROW_GAP: f32 = 3.0;
const PADDING: f32 = 6.0;
/// Body height of a node that shows a symbol
const SYMBOL_HEIGHT: f32 = 40.0;
/// Below this zoom the fields are too small to use, so the value is drawn as text
const MIN_EDIT_ZOOM: f32 = 0.45;
const MAX_EDIT_ZOOM: f32 = 1.6;

/// The sign or short word that stands for `operation` on its node
pub(super) fn symbol(operation: &str) -> &'static str {
    match operation {
        "add" => "+",
        "sub" => "\u{2212}",
        "mul" => "\u{d7}",
        "mulscalar" => "k\u{d7}",
        "div" => "\u{f7}",
        "neg" => "\u{2212}a",
        "mod" => icon::PERCENT,
        "dot" => "a\u{b7}b",
        "and" => "AND",
        "or" => "OR",
        "not" => "NOT",
        "eq" => icon::EQUALS,
        "noteq" => icon::NOT_EQUALS,
        "gt" => icon::GREATER_THAN,
        "gteq" => icon::GREATER_THAN_OR_EQUAL,
        "lr" => icon::LESS_THAN,
        "lreq" => icon::LESS_THAN_OR_EQUAL,
        "abs" => "|x|",
        "floor" => "floor",
        "ceil" => "ceil",
        "cos" => "cos",
        "sin" => "sin",
        "tan" => "tan",
        "atan2" => "atan2",
        "cartesian" => "polar",
        "sqrt" => icon::RADICAL,
        "log" => "log",
        "exp" => "exp",
        "log2" => "log2",
        "pow2" => "2^x",
        "pow" => "a^b",
        "lerp" => "lerp",
        "min" => "min",
        "max" => "max",
        "rand" => icon::DICE_FIVE,
        "sequence" => icon::LIST_NUMBERS,
        "ifelse" => "if",
        "while" => icon::REPEAT,
        "vector2" | "ivector2" => "xy",
        "vector3" | "ivector3" => "xyz",
        "vector4" | "ivector4" => "xyzw",
        "tofloat" => "float",
        "tofloat2" => "float2",
        "tofloat3" => "float3",
        "tofloat4" => "float4",
        "toint1" => "int",
        "toint2" => "int2",
        "toint3" => "int3",
        "toint4" => "int4",
        "samplelum" => icon::EYEDROPPER,
        "samplecol" => icon::EYEDROPPER_SAMPLE,
        _ => icon::FUNCTION,
    }
}

/// Graph-space height the body of `node` needs for its value fields or its symbol
pub(super) fn height(node: &FunctionNode) -> f32 {
    match &node.value {
        Some(value) if node.missing.is_none() => {
            let rows = crate::value_editor::rows(value) as f32;
            rows * ROW_HEIGHT + (rows - 1.0) * ROW_GAP + PADDING * 2.0
        }
        _ => SYMBOL_HEIGHT,
    }
}

/// Whether the value of a node is edited on its face at `zoom`; it is drawn as text otherwise
pub(super) fn editable(zoom: f32) -> bool {
    zoom >= MIN_EDIT_ZOOM
}

/// The value fields of a node inside `body`, sized with the canvas zoom; true when `value` changed
pub(super) fn edit(ui: &mut Ui, body: Rect, clip: Rect, zoom: f32, value: &mut Value) -> bool {
    let scale = zoom.clamp(MIN_EDIT_ZOOM, MAX_EDIT_ZOOM);
    let content = body.shrink(PADDING * zoom);
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(content));
    child.set_clip_rect(content.expand(1.0).intersect(clip));

    let style = child.style_mut();
    style.override_font_id = Some(egui::FontId::proportional(12.0 * scale));
    style.spacing.interact_size = vec2(24.0 * scale, ROW_HEIGHT * zoom);
    style.spacing.item_spacing = vec2(4.0 * scale, ROW_GAP * zoom);
    style.spacing.button_padding = vec2(4.0 * scale, 0.0);
    style.spacing.icon_width = 14.0 * scale;
    style.spacing.icon_width_inner = 8.0 * scale;
    style.spacing.icon_spacing = 4.0 * scale;

    crate::value_editor::show_stacked(&mut child, value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use materialcraft_engine::nodes::function::{FUNCTIONS, default_value};

    fn node(operation: &str) -> FunctionNode {
        FunctionNode {
            id: 1,
            type_code: 0,
            op: operation.into(),
            missing: None,
            missing_data: String::new(),
            instance: None,
            source: None,
            inputs: Vec::new(),
            value: default_value(operation),
            pos: [0.0; 2],
        }
    }

    #[test]
    fn every_operation_without_a_value_has_its_own_symbol() {
        for definition in FUNCTIONS {
            if definition.id == "passthrough" || default_value(definition.id).is_some() {
                continue;
            }

            assert_ne!(symbol(definition.id), icon::FUNCTION, "{} has no symbol", definition.id);
        }
    }

    #[test]
    fn a_body_grows_with_the_components_of_its_value() {
        let single = height(&node("const_float1"));
        assert_eq!(height(&node("const_float4")), single + 3.0 * (ROW_HEIGHT + ROW_GAP));
        assert_eq!(height(&node("add")), SYMBOL_HEIGHT);

        let mut unavailable = node("const_float4");
        unavailable.missing = Some(Vec::new());
        assert_eq!(height(&unavailable), SYMBOL_HEIGHT);
    }
}

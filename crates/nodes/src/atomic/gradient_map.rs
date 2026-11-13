use super::ATOMIC;
use crate::{CookContext, CookResult, NodeDefinition, node};
use materialcraft_core::{Gradient, OutputFormat};
use materialcraft_image::{Kind, luminance};

pub(super) fn definition() -> NodeDefinition {
    node("gradient_map", "Gradient map", ATOMIC)
        .desc("Maps grayscale values to colors along a gradient.")
        .atomic()
        .gray_in("input", "Input")
        .gradient("gradient", "Gradient", Gradient::default())
        .boolean("color_mode", "Color output", true)
        .any_out()
        .cook(gradient_map)
}

fn gradient_map(context: &CookContext) -> CookResult {
    let input = context.gray_or(0, 0.0);
    let g = context.gradient("gradient");
    if matches!(context.format, OutputFormat::Inherit | OutputFormat::U16) {
        return Ok(vec![integer_map(&input, g, !context.b("color_mode"))]);
    }
    let lut: Vec<[f32; 4]> = (0..=512).map(|i| g.eval(i as f32 / 512.0)).collect();
    let gray = !context.b("color_mode");
    let look = |v: f32| lut.get((v.clamp(0.0, 1.0) * 512.0).round() as usize).copied().unwrap_or([0.0, 0.0, 0.0, 1.0]);
    Ok(vec![if gray { input.map(Kind::Gray, |p| [luminance(look(p[0])), 0.0, 0.0, 0.0]) } else { input.map(Kind::Color, |p| look(p[0])) }])
}

fn integer_map(input: &materialcraft_image::Image, mut gradient: Gradient, gray: bool) -> materialcraft_image::Image {
    // The 16-bit path evaluates a 256-entry table in integer position space.
    // Keeping normalized positions or sampling a coarse table by nearest value
    // shifts narrow transitions and changes the subsequent noise octaves.
    for key in &mut gradient.keys {
        key.pos = (key.pos.clamp(0.0, 1.0) * 65535.0).round();
    }
    let table: Vec<[u32; 4]> =
        (0..256).map(|index| gradient.eval(index as f32 * 257.0).map(|value| (value.clamp(0.0, 1.0) * 65535.0).round() as u32)).collect();
    let lookup = |value: f32| {
        let value = if value.is_finite() { value.clamp(0.0, 1.0) } else { 0.0 };
        // The half-texel conversion precedes eight-bit interpolation weights;
        // its float rounding is visible at the middle of a 16-bit ramp.
        let position = (value * 255.0 + 0.5) - 0.5;
        let fixed = (position * 256.0).round() as usize;
        let index = fixed / 256;
        let weight = (fixed % 256) as u32;
        let left = table.get(index).copied().unwrap_or([0; 4]);
        let right = table.get(index.saturating_add(1)).copied().unwrap_or(left);
        std::array::from_fn(|channel| ((left[channel] * (256 - weight) + right[channel] * weight + 128) >> 8) as f32 / 65535.0)
    };
    if gray { input.map(Kind::Gray, |pixel| [luminance(lookup(pixel[0])), 0.0, 0.0, 0.0]) } else { input.map(Kind::Color, |pixel| lookup(pixel[0])) }
}

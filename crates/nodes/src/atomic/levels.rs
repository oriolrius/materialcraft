use super::ATOMIC;
use crate::{CookContext, CookResult, NodeDefinition, node};
use materialcraft_image::Kind;

pub(super) fn definition() -> NodeDefinition {
    node("levels", "Levels", ATOMIC)
        .desc("Remaps the input range, midtones and output range.")
        .atomic()
        .any_in("input", "Input")
        .float4("in_low", "Level in low", [0.0; 4], 0.0, 1.0)
        .float4("in_mid", "Level in mid", [0.5; 4], 0.0, 1.0)
        .float4("in_high", "Level in high", [1.0; 4], 0.0, 1.0)
        .float4("out_low", "Level out low", [0.0; 4], 0.0, 1.0)
        .float4("out_high", "Level out high", [1.0; 4], 0.0, 1.0)
        .any_out()
        .cook(levels)
}

fn levels(context: &CookContext) -> CookResult {
    let image = context.any_or(0, Kind::Gray, [0.0; 4]);
    let low = context.v4("in_low");
    let mid = context.v4("in_mid");
    let high = context.v4("in_high");
    let output_low = context.v4("out_low");
    let output_high = context.v4("out_high");

    Ok(vec![image.map(image.kind, |pixel| {
        std::array::from_fn(|channel| levels_fn(pixel[channel], low[channel], mid[channel], high[channel], output_low[channel], output_high[channel]))
    })])
}

pub fn levels_fn(v: f32, in_low: f32, in_mid: f32, in_high: f32, out_low: f32, out_high: f32) -> f32 {
    let span = in_high - in_low;
    let t = if span.abs() < 1e-6 { if v >= in_high { 1.0 } else { 0.0 } } else { ((v - in_low) / span).clamp(0.0, 1.0) };
    // NaN (from an input or a hostile parameter) would otherwise spread through the graph
    let t = if t.is_finite() { t } else { 0.0 };
    let mid = if in_mid.is_finite() { in_mid.clamp(0.0, 1.0) } else { 0.5 };
    // The midpoint slider controls reciprocal gamma below neutral and gamma above it.
    // Treating it as an input value that maps to 0.5 changes authored levels curves.
    let g = if mid <= 0.5 { 1.0 / (9.0 - 16.0 * mid) } else { 16.0 * mid - 7.0 };
    out_low + t.powf(g) * (out_high - out_low)
}

#[cfg(test)]
mod levels_tests {
    use super::levels_fn;

    #[test]
    fn levels_remaps_the_input_range_midtones_and_output_range() {
        let close = |a: f32, b: f32| (a - b).abs() < 1e-5;
        // Identity by default
        for v in [0.0, 0.25, 0.5, 1.0] {
            assert!(close(levels_fn(v, 0.0, 0.5, 1.0, 0.0, 1.0), v));
        }
        // The input range stretches and clips
        assert!(close(levels_fn(0.4, 0.2, 0.5, 0.6, 0.0, 1.0), 0.5));
        assert!(close(levels_fn(0.1, 0.2, 0.5, 0.6, 0.0, 1.0), 0.0));
        assert!(close(levels_fn(0.9, 0.2, 0.5, 0.6, 0.0, 1.0), 1.0));
        // The same gamma curve applies after normalizing the input range.
        assert!(close(levels_fn(0.25, 0.0, 0.25, 1.0, 0.0, 1.0), 0.7578583));
        assert!(close(levels_fn(0.3, 0.2, 0.25, 0.6, 0.0, 1.0), 0.7578583));
        // The output range, inverted when low > high
        assert!(close(levels_fn(0.5, 0.0, 0.5, 1.0, 0.2, 0.6), 0.4));
        assert!(close(levels_fn(0.0, 0.0, 0.5, 1.0, 1.0, 0.0), 1.0));
        // A collapsed input range is a threshold; NaN never escapes
        assert_eq!(levels_fn(0.49, 0.5, 0.5, 0.5, 0.0, 1.0), 0.0);
        assert_eq!(levels_fn(0.5, 0.5, 0.5, 0.5, 0.0, 1.0), 1.0);
        assert!(levels_fn(f32::NAN, 0.0, 0.5, 1.0, 0.0, 1.0).is_finite());
        assert!(levels_fn(0.5, f32::NAN, 0.5, f32::NAN, 0.0, 1.0).is_finite());
    }
}

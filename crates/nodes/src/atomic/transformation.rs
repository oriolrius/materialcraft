use super::ATOMIC;
use crate::{CookContext, CookResult, NodeDefinition, node};
use materialcraft_core::OutputFormat;
use materialcraft_image::{Image, Kind};

pub(super) fn definition() -> NodeDefinition {
    node("transformation", "Transformation 2D", ATOMIC)
        .desc("Transforms the input with an offset and a 2 by 2 matrix.")
        .atomic()
        .any_in("input", "Input")
        .float2("offset", "Offset", [0.0; 2], -1.0, 1.0)
        .float4("matrix", "Transform matrix", [1.0, 0.0, 0.0, 1.0], -10.0, 10.0)
        .choice("filtering", "Filtering", &["Bilinear", "Nearest"], 0)
        .color("matte", "Matte color", [0.0; 4])
        .any_out()
        .cook(transformation)
}

fn transformation(context: &CookContext) -> CookResult {
    // Preserve source texels so nearest sampling does not blur the input
    let input = context.input(0).cloned().unwrap_or_else(|| Image::filled(context.w, context.h, Kind::Gray, [0.0; 4]));
    let offset = context.v2("offset");
    let matrix = context.v4("matrix");
    let determinant = matrix[0] * matrix[3] - matrix[1] * matrix[2];
    if !determinant.is_finite() || determinant.abs() < 1e-8 {
        return Err("transform matrix is singular".into());
    }

    let nearest = context.i("filtering") == 1;
    let input = if nearest { nearest_mip(input, matrix, context) } else { input };
    let matte = context.v4("matte");

    // Keep independent wrap axes for panoramic images
    let tiling = context.vars.get("$tiling").and_then(materialcraft_core::Value::as_i32).map_or([context.tile; 2], |mask| [mask & 1 != 0, mask & 2 != 0]);

    Ok(vec![Image::generate_uv(context.w, context.h, input.kind, |u, v| {
        let x = u - 0.5;
        let y = v - 0.5;

        // SBS stores matrix columns consecutively; interpreting rows reverses rotations
        let source_u = matrix[0] * x + matrix[2] * y + 0.5 + offset[0];
        let source_v = matrix[1] * x + matrix[3] * y + 0.5 + offset[1];
        if (!tiling[0] && !(0.0..1.0).contains(&source_u)) || (!tiling[1] && !(0.0..1.0).contains(&source_v)) {
            matte
        } else if nearest {
            input.sample_nearest_axes(source_u, source_v, tiling)
        } else {
            input.sample_axes(source_u, source_v, tiling)
        }
    })])
}

fn nearest_mip(mut input: Image, matrix: [f32; 4], context: &CookContext) -> Image {
    let width = context.w.max(1) as f32;
    let height = context.h.max(1) as f32;
    let dx = (matrix[0] * input.w as f32 / width).hypot(matrix[1] * input.h as f32 / width);
    let dy = (matrix[2] * input.w as f32 / height).hypot(matrix[3] * input.h as f32 / height);

    // Thirteen reductions cover the maximum supported side of 8192 texels
    let levels = dx.max(dy).max(1.0).log2().round().clamp(0.0, 13.0) as u32;
    let maximum = match context.format {
        OutputFormat::U8 => Some(255.0),
        OutputFormat::Inherit | OutputFormat::U16 => Some(65535.0),
        OutputFormat::F16 | OutputFormat::F32 => None,
    };

    for _ in 0..levels {
        if input.w <= 1 && input.h <= 1 {
            break;
        }

        // Integer mip levels round four-texel averages halfway up
        input = Image::generate(input.w.div_ceil(2), input.h.div_ceil(2), input.kind, |x, y| {
            let (x, y) = (i64::from(x) * 2, i64::from(y) * 2);
            let pixels = [input.get(x, y, false), input.get(x + 1, y, false), input.get(x, y + 1, false), input.get(x + 1, y + 1, false)];
            std::array::from_fn(|channel| {
                if let Some(maximum) = maximum {
                    let sum: u32 = pixels.iter().map(|pixel| (pixel[channel].clamp(0.0, 1.0) * maximum).round() as u32).sum();
                    ((sum + 2) / 4) as f32 / maximum
                } else {
                    pixels.iter().map(|pixel| pixel[channel]).sum::<f32>() * 0.25
                }
            })
        });
    }

    input
}

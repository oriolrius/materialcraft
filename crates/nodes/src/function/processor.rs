use super::{EvaluationContext, EvaluationState, compile};
use crate::{CookContext, CookResult};
use materialcraft_core::Value;
use materialcraft_image::{Image, Kind};
use rayon::prelude::*;
use std::sync::Arc;

fn pixel(value: Value, gray: bool, format: materialcraft_core::OutputFormat) -> Result<[f32; 4], String> {
    let values = super::value_type::floats(&value)?;
    let integer = matches!(format, materialcraft_core::OutputFormat::U8 | materialcraft_core::OutputFormat::U16 | materialcraft_core::OutputFormat::Inherit);
    // Integer texture storage converts NaN to black and saturates infinities.
    // Rejecting those values before storage breaks zero-softness masks.
    if !integer && values.iter().take(super::value_type::dimensions(&value)).any(|value| !value.is_finite()) {
        return Err("pixel function produced a non-finite output".into());
    }
    Ok(if gray {
        let intensity = if super::value_type::dimensions(&value) == 4 {
            // RGBA results on a grayscale processor use equal RGB weights.
            // Preserve the fused accumulation order at floating-point outputs.
            values[2].mul_add(1.0 / 3.0, values[0].mul_add(1.0 / 3.0, values[1] * (1.0 / 3.0)))
        } else {
            values[0]
        };
        [intensity, 0.0, 0.0, 0.0]
    } else {
        match super::value_type::dimensions(&value) {
            1 => [values[0], values[0], values[0], 1.0],
            2 => [values[0], values[1], 0.0, 1.0],
            3 => [values[0], values[1], values[2], 1.0],
            _ => values,
        }
    })
}

pub(crate) fn pixel_processor(context: &CookContext) -> CookResult {
    let graph = context.function("function").ok_or("missing pixel function")?;
    let program = compile(graph, context.seed)?;
    let gray = !context.b("color_mode");
    let kind = if gray { Kind::Gray } else { Kind::Color };

    let (width, height) = materialcraft_image::clamp_size(context.w, context.h);
    let channels = kind.channels();
    let mut data = vec![0.0; width as usize * height as usize * channels];
    data.par_chunks_mut(width as usize * channels).enumerate().try_for_each_init(EvaluationState::default, |state, (y, row)| -> Result<(), String> {
        for (x, target) in row.chunks_exact_mut(channels).enumerate() {
            let evaluation = EvaluationContext {
                scope: super::FunctionScope::PixelProcessor,
                position: [(x as f32 + 0.5) / width as f32, (y as f32 + 0.5) / height as f32],
                size: [width as f32, height as f32],
                images: context.inputs,
                tile: context.tile,
                variables: context.vars,
            };

            let result = pixel(program.evaluate_pixel(&evaluation, state)?, gray, context.format)?;
            target.copy_from_slice(result.get(..channels).ok_or("invalid output channels")?);
        }
        Ok(())
    })?;
    Ok(vec![Image { w: width, h: height, kind, integer_sampling: false, data: Arc::new(data) }])
}

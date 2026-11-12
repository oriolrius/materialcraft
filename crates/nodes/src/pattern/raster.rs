use materialcraft_image::{Image, Kind};
use std::sync::Arc;

use super::{pattern_stamp::PatternStamp, region::Region, shape};
use crate::CookContext;

const MAX_SAMPLES: u64 = 256_000_000;

pub(super) fn render(context: &CookContext, stamps: &[PatternStamp], max_depth: u32) -> Result<Image, String> {
    let (width, height) = materialcraft_image::clamp_size(context.w, context.h);
    let kind = if context.b("color_mode") { Kind::Color } else { Kind::Gray };
    let mut image =
        context.input(0).map(|image| image.resize(width, height, context.tile)).unwrap_or_else(|| Image::filled(width, height, kind, context.v4("background")));
    image = if kind == Kind::Gray { image.to_gray() } else { image.to_color() };
    if stamps.is_empty() {
        return Ok(image);
    }
    let channels = kind.channels();
    let opacity = context.f("opacity").max(0.0);
    let weights = depth_weights(context.f("roughness"), max_depth)?;
    let pixels = Arc::make_mut(&mut image.data);
    let mut budget = MAX_SAMPLES;
    let mut copies_left = 1_048_576_u64;
    let render_region = Region::new(context.v4("render_region"))?;
    let period = if context.tile { Region::new(context.v4("tiling_region"))?.period() } else { [1.0; 2] };

    for stamp in stamps {
        if !stamp.center.iter().chain(&stamp.size).chain(&stamp.color).chain([&stamp.rotation, &stamp.variation]).all(|value| value.is_finite()) {
            return Err("pattern parameters must be finite".into());
        }
        if stamp.size.iter().any(|size| size.abs() < 1e-8) {
            continue;
        }
        if !matches!(stamp.blend, 0..=2) {
            return Err(format!("unsupported pattern blending mode {}", stamp.blend));
        }
        if !(2..=18).contains(&stamp.pattern) && !(256..256 + crate::MAX_IMAGE_INPUTS as i32).contains(&stamp.pattern) {
            return Err(format!("unsupported pattern {}", stamp.pattern));
        }

        let opacity = opacity * weights.get(stamp.depth as usize).copied().ok_or("invalid pattern depth")?;
        if !opacity.is_finite() {
            return Err("pattern roughness overflowed its depth opacity".into());
        }

        let (sine, cosine) = (stamp.rotation * std::f32::consts::TAU).sin_cos();
        let radius = [
            (stamp.size[0] * cosine).abs() * 0.5 + (stamp.size[1] * sine).abs() * 0.5,
            (stamp.size[0] * sine).abs() * 0.5 + (stamp.size[1] * cosine).abs() * 0.5,
        ];
        let center = if context.tile { std::array::from_fn(|axis| stamp.center[axis].rem_euclid(period[axis])) } else { stamp.center };
        let repeats = if context.tile { render_region.repeats(center, radius, period)? } else { [[0, 0]; 2] };
        let count = |axis: usize| (i64::from(repeats[axis][1]) - i64::from(repeats[axis][0]) + 1).max(0) as u64;
        let copies = count(0).checked_mul(count(1)).ok_or("pattern repetition exceeds its budget")?;
        copies_left = copies_left.checked_sub(copies).ok_or("pattern repetition exceeds its budget")?;
        for repeat_y in repeats[1][0]..=repeats[1][1] {
            for repeat_x in repeats[0][0]..=repeats[0][1] {
                let center = [center[0] + repeat_x as f32 * period[0], center[1] + repeat_y as f32 * period[1]];
                let [[left, right], [top, bottom]] = render_region.bounds(center, radius, [width, height]);
                let samples = u64::from(right.saturating_sub(left)) * u64::from(bottom.saturating_sub(top));
                budget = budget.checked_sub(samples).ok_or("pattern rendering exceeded its pixel budget")?;
                for y in top..bottom {
                    for x in left..right {
                        let point = render_region.point([x, y], [width, height]);
                        let delta = [point[0] - center[0], point[1] - center[1]];
                        let local = [(delta[0] * cosine + delta[1] * sine) * 2.0 / stamp.size[0], (-delta[0] * sine + delta[1] * cosine) * 2.0 / stamp.size[1]];
                        // Half-open bounds give shared edges one owner, including subpixel quadrants.
                        // Including both sides can add four adjacent stamps to the same pixel.
                        if local.iter().any(|coordinate| *coordinate < -1.0 || *coordinate >= 1.0) {
                            continue;
                        }

                        let pattern = if stamp.pattern >= 256 {
                            let index = (stamp.pattern - 256) as usize + 1;
                            match context.input(index) {
                                Some(input) => {
                                    let uv = local.map(|coordinate| coordinate * 0.5 + 0.5);
                                    if stamp.filtering == 0 {
                                        input.get((uv[0] * input.w as f32) as i64, (uv[1] * input.h as f32) as i64, false)
                                    } else {
                                        input.sample(uv[0], uv[1], false)
                                    }
                                }
                                None => [0.0; 4],
                            }
                        } else {
                            let coverage = shape::coverage(stamp.pattern, local[0], local[1], stamp.variation)?;
                            [coverage, coverage, coverage, coverage]
                        };

                        let pixel = (y as usize * width as usize + x as usize) * channels;
                        let target = pixels.get_mut(pixel..pixel + channels).ok_or("invalid pattern pixel")?;
                        for (channel, target) in target.iter_mut().enumerate() {
                            let image_alpha = if stamp.pattern >= 256 && !stamp.premultiplied && channel < 3 { pattern[3] } else { 1.0 };
                            let alpha = image_alpha * if kind == Kind::Color && channel < 3 { stamp.color[3] } else { 1.0 };
                            let value = pattern[channel] * stamp.color[channel] * alpha * opacity;
                            *target = match stamp.blend {
                                // Grayscale legacy maximum blends have no alpha channel to composite.
                                1 if kind == Kind::Gray => target.max(value),
                                1 => value + *target * (1.0 - (pattern[3] * stamp.color[3] * opacity).clamp(0.0, 1.0)),
                                2 => target.max(value),
                                _ => *target + value,
                            };
                            if !target.is_finite() {
                                return Err("pattern accumulation overflowed".into());
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(image)
}

fn depth_weights(roughness: f32, max_depth: u32) -> Result<Vec<f32>, String> {
    // Traversal is bounded by the 4096-node acyclic pattern graph. Keep this allocation bounded too.
    if max_depth >= 4096 || !roughness.is_finite() || roughness < 0.0 {
        return Err("pattern roughness must be finite and nonnegative, with depth below 4096".into());
    }
    let mut weights = vec![1.0_f64; max_depth as usize + 1];
    // Zero is the native no-attenuation mode, not the limit of the normalized geometric weights.
    if roughness == 0.0 {
        return Ok(weights.into_iter().map(|value| value as f32).collect());
    }
    // Normalize r^depth over all visited quadrant depths, including non-drawing ancestors.
    // Build from the largest weight so large roughness/depth cannot overflow the geometric series.
    let mut weight = 1.0;
    let ratio = f64::from(roughness);
    if roughness <= 1.0 {
        for value in &mut weights {
            *value = weight;
            weight *= ratio;
        }
    } else {
        for value in weights.iter_mut().rev() {
            *value = weight;
            weight /= ratio;
        }
    }
    let sum: f64 = weights.iter().sum();
    Ok(weights.into_iter().map(|value| (value / sum) as f32).collect())
}

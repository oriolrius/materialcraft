//! Original, deterministic resource recipes. This module never opens source assets.
use super::Result;
use image::{DynamicImage, ImageBuffer, ImageFormat, Luma, Rgba};
use std::io::Cursor;

pub struct Recipe {
    pub key: &'static str,
    pub path: &'static str,
    pub extension: &'static str,
    pub description: &'static str,
}

const RECIPES: &[Recipe] = &[
    Recipe {
        key: "perm",
        path: "resources/perm.png",
        extension: "png",
        description: "Sixteen independently shuffled 1024-entry permutation rows. Original seeded lookup substitute; external sampling semantics require validation.",
    },
    Recipe {
        key: "bluenoise_256",
        path: "resources/bluenoise_256.png",
        extension: "png",
        description: "Original ranked, toroidal high-pass noise for grayscale dithering and sampling.",
    },
    Recipe {
        key: "bluenoise_256_c",
        path: "resources/bluenoise_256_c.png",
        extension: "png",
        description: "Four independent original ranked high-pass noise channels.",
    },
    Recipe {
        key: "scratched_metal",
        path: "resources/scratched_metal.png",
        extension: "png",
        description: "Original periodic scratch mask with fine directional grooves and uneven wear.",
    },
    Recipe { key: "leather", path: "resources/leather.png", extension: "png", description: "Original cellular grain relief with irregular pebble boundaries." },
    Recipe {
        key: "scratch_damage",
        path: "resources/scratch_damage.png",
        extension: "png",
        description: "Original crossed scratch mask with sparse intersecting cuts.",
    },
    Recipe {
        key: "shuf4_64_32",
        path: "resources/shuf4_64_32.exr",
        extension: "exr",
        description: "Four original shuffled stratified sequences in a 64 by 32 floating-point lookup. External sampling semantics require validation.",
    },
    Recipe {
        key: "sun",
        path: "resources/sun.exr",
        extension: "exr",
        description: "Original 512 by 16 analytical RGB extinction table varying optical path and haze. External sampling semantics require validation.",
    },
    Recipe {
        key: "moss01_bmp",
        path: "resources/moss01_bmp.bmp",
        extension: "bmp",
        description: "Original periodic multiscale moss coverage mask with fine clustered detail.",
    },
    Recipe {
        key: "courier_new",
        path: "resources/cour.ttf",
        extension: "ttf",
        description: "Original monospaced segment font for numerical graph labels, generated from geometric strokes.",
    },
    Recipe {
        key: "DoorFrame",
        path: "resources/DoorFrame.svg",
        extension: "svg",
        description: "Original vector arch frame constructed from a rounded opening and straight jambs.",
    },
];

pub fn recipe(key: &str) -> Result<&'static Recipe> {
    RECIPES.iter().find(|recipe| recipe.key == key).ok_or_else(|| format!("no original resource recipe for `{key}`").into())
}

struct Random(u64);
impl Random {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

fn permutation(length: usize, seed: u64) -> Vec<usize> {
    let mut values: Vec<_> = (0..length).collect();
    let mut random = Random(seed);
    for index in (1..length).rev() {
        let other = (random.next() % (index as u64 + 1)) as usize;
        values.swap(index, other);
    }
    values
}

/// Alternate a periodic high-pass filter and rank normalization. The exact histogram remains
/// uniform; negative short-range correlation suppresses low-frequency fluctuations.
fn ranked_noise(seed: u64) -> Vec<u16> {
    const SIDE: usize = 256;
    let mut field: Vec<f32> = permutation(SIDE * SIDE, seed).into_iter().map(|v| v as f32 / 65535.0).collect();
    let mut horizontal = vec![0.0; field.len()];
    let mut filtered = vec![0.0; field.len()];
    let weights = [(254, 1.0), (255, 4.0), (0, 6.0), (1, 4.0), (2, 1.0)];
    for _ in 0..6 {
        for y in 0..SIDE {
            for x in 0..SIDE {
                horizontal[y * SIDE + x] = weights.iter().map(|(dx, w)| field[y * SIDE + (x + dx) % SIDE] * w / 16.0).sum();
            }
        }
        for y in 0..SIDE {
            for x in 0..SIDE {
                let blurred: f32 = weights.iter().map(|(dy, w)| horizontal[((y + dy) % SIDE) * SIDE + x] * w / 16.0).sum();
                filtered[y * SIDE + x] = field[y * SIDE + x] - 0.85 * blurred;
            }
        }
        let mut order: Vec<_> = (0..field.len()).collect();
        order.sort_by(|a, b| filtered[*a].total_cmp(&filtered[*b]).then(a.cmp(b)));
        for (rank, index) in order.into_iter().enumerate() {
            field[index] = rank as f32 / 65535.0;
        }
    }
    field.into_iter().map(|value| (value * 65535.0).round() as u16).collect()
}

fn hash(x: i32, y: i32, cells: i32, seed: u32) -> f32 {
    let mut n = (x.rem_euclid(cells) as u32).wrapping_mul(0x9e3779b9) ^ (y.rem_euclid(cells) as u32).wrapping_mul(0x85ebca6b) ^ seed;
    n ^= n >> 16;
    n = n.wrapping_mul(0x7feb352d);
    n ^= n >> 15;
    n = n.wrapping_mul(0x846ca68b);
    n ^= n >> 16;
    (n >> 8) as f32 / 16_777_215.0
}

fn noise(x: f32, y: f32, cells: i32, seed: u32) -> f32 {
    let (x, y) = (x * cells as f32, y * cells as f32);
    let (ix, iy) = (x.floor() as i32, y.floor() as i32);
    let smooth = |v: f32| v * v * (3.0 - 2.0 * v);
    let (tx, ty) = (smooth(x - x.floor()), smooth(y - y.floor()));
    let mix = |a: f32, b: f32, t: f32| a + (b - a) * t;
    mix(mix(hash(ix, iy, cells, seed), hash(ix + 1, iy, cells, seed), tx), mix(hash(ix, iy + 1, cells, seed), hash(ix + 1, iy + 1, cells, seed), tx), ty)
}

fn cellular(x: f32, y: f32) -> f32 {
    let (x, y) = (x * 24.0, y * 24.0);
    let (ix, iy) = (x.floor() as i32, y.floor() as i32);
    let (mut first, mut second) = (f32::INFINITY, f32::INFINITY);
    for dy in -1..=1 {
        for dx in -1..=1 {
            let (cx, cy) = (ix + dx, iy + dy);
            let px = cx as f32 + 0.15 + hash(cx, cy, 24, 73) * 0.7;
            let py = cy as f32 + 0.15 + hash(cx, cy, 24, 131) * 0.7;
            let d = (x - px).hypot(y - py);
            if d < first {
                second = first;
                first = d;
            } else if d < second {
                second = d;
            }
        }
    }
    ((second - first) * 8.0).clamp(0.0, 1.0)
}

fn mask(key: &str) -> DynamicImage {
    let image = ImageBuffer::<Luma<u16>, _>::from_fn(512, 512, |x, y| {
        let (u, v) = (x as f32 / 512.0, y as f32 / 512.0);
        let grain = noise(u, v, 128, 701);
        let value = match key {
            "leather" => 0.12 + 0.65 * cellular(u, v) + 0.15 * grain + 0.08 * noise(u, v, 8, 19),
            "moss01_bmp" => {
                let coverage = noise(u, v, 8, 31) * 0.5 + noise(u, v, 24, 61) * 0.3 + grain * 0.2;
                ((coverage - 0.35) * 2.8).clamp(0.0, 1.0) * (0.6 + 0.4 * grain)
            }
            _ => {
                let phase = std::f32::consts::TAU * (u * 87.0 + v * 3.0 + noise(u, v, 8, 47) * 1.4);
                let groove = phase.sin().abs().powf(30.0) * noise(u, v, 16, 83);
                let cross = (std::f32::consts::TAU * (u * 31.0 - v * 53.0 + noise(u, v, 4, 59))).sin().abs().powf(60.0);
                if key == "scratch_damage" { (groove * 1.4 + cross * 0.55).clamp(0.0, 1.0) } else { 0.35 + grain * 0.35 + groove * 0.3 }
            }
        };
        Luma([(value.clamp(0.0, 1.0) * 65535.0).round() as u16])
    });
    DynamicImage::ImageLuma16(image)
}

pub fn generate(key: &str) -> Result<Vec<u8>> {
    recipe(key)?;
    if key == "courier_new" {
        return super::font::generate();
    }
    if key == "DoorFrame" {
        return Ok(br#"<svg xmlns="http://www.w3.org/2000/svg" width="512" height="512" viewBox="0 0 512 512"><path fill="white" fill-rule="evenodd" d="M88 480V220C88 38 424 38 424 220V480H370V220C370 109 142 109 142 220V480Z"/></svg>"#.to_vec());
    }
    let image = match key {
        "perm" => {
            let values: Vec<u16> = (0..16)
                .flat_map(|row| permutation(1024, 0x6d6174657269616c ^ (row * 313 + 1)).into_iter().map(|v| ((v as f32 / 1023.0) * 65535.0).round() as u16))
                .collect();
            DynamicImage::ImageLuma16(ImageBuffer::from_raw(1024, 16, values).ok_or("invalid permutation image")?)
        }
        "bluenoise_256" => DynamicImage::ImageLuma16(ImageBuffer::from_raw(256, 256, ranked_noise(0x123456789abcdef)).ok_or("invalid ranked noise image")?),
        "bluenoise_256_c" => {
            let channels: Vec<_> = (0..4).map(|c| ranked_noise(0x1729abcd + c * 101)).collect();
            DynamicImage::ImageRgba16(ImageBuffer::from_fn(256, 256, |x, y| Rgba(std::array::from_fn(|c| channels[c][(y * 256 + x) as usize]))))
        }
        "shuf4_64_32" => {
            let channels: Vec<_> = (0..4).map(|c| permutation(2048, 0x27ab1395 + c * 1009)).collect();
            DynamicImage::ImageRgba32F(ImageBuffer::from_fn(64, 32, |x, y| {
                Rgba(std::array::from_fn(|c| (channels[c][(y * 64 + x) as usize] as f32 + 0.5) / 2048.0))
            }))
        }
        "sun" => DynamicImage::ImageRgba32F(ImageBuffer::from_fn(512, 16, |x, y| {
            let optical_path = 1.0 / (0.03 + (x as f32 + 0.5) / 512.0);
            let haze = 0.25 + y as f32 / 5.0;
            let channel = |extinction: f32| (-optical_path * (extinction + haze * 0.045)).exp();
            Rgba([channel(0.06), channel(0.12), channel(0.24), 1.0])
        })),
        _ => mask(key),
    };
    let format = match recipe(key)?.extension {
        "exr" => ImageFormat::OpenExr,
        "bmp" => ImageFormat::Bmp,
        _ => ImageFormat::Png,
    };
    let image = if format == ImageFormat::Bmp { DynamicImage::ImageLuma8(image.into_luma8()) } else { image };
    let mut output = Cursor::new(Vec::new());
    image.write_to(&mut output, format)?;
    Ok(output.into_inner())
}

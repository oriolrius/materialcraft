//! Original procedural recipes and attributed library resources; never reads source assets.
use super::Result;
use image::{DynamicImage, ImageBuffer, ImageFormat};
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
        description: "256 by 256 grayscale blue noise for dithering and sampling.",
    },
    Recipe {
        key: "bluenoise_256_c",
        path: "resources/bluenoise_256_c.exr",
        extension: "exr",
        description: "256 by 256 color blue noise for multichannel sampling.",
    },
    Recipe { key: "scratched_metal", path: "resources/scratched_metal.png", extension: "png", description: "Scratched metal texture for surface wear." },
    Recipe { key: "leather", path: "resources/leather.png", extension: "png", description: "Leather grain texture for surface detail." },
    Recipe { key: "scratch_damage", path: "resources/scratch_damage.png", extension: "png", description: "Scratch damage texture for worn surfaces." },
    Recipe { key: "shuf4_64_32", path: "resources/shuf4_64_32.exr", extension: "exr", description: "Four-channel 64 by 32 floating-point sampling lookup." },
    Recipe { key: "sun", path: "resources/sun.exr", extension: "exr", description: "512 by 16 solar coefficient lookup. All RGBA channels are numeric data." },
    Recipe { key: "moss01_bmp", path: "resources/moss01_bmp.bmp", extension: "bmp", description: "Moss texture for coverage and surface detail." },
    Recipe {
        key: "courier_new",
        path: "resources/LiberationMono-Regular.ttf",
        extension: "ttf",
        description: "Liberation Mono Regular fallback from craft-fonts for missing point-preview fonts.",
    },
    Recipe {
        key: "DoorFrame",
        path: "resources/DoorFrame.svg",
        extension: "svg",
        description: "Original vector arch frame constructed from a rounded opening and straight jambs.",
    },
];

pub fn recipe(key: &str) -> Result<&'static Recipe> {
    RECIPES.iter().find(|recipe| recipe.key == key).ok_or_else(|| format!("no resource recipe for `{key}`").into())
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

pub fn load(key: &str) -> Result<Vec<u8>> {
    let recipe = recipe(key)?;
    if !notices(key).is_empty() {
        return Ok(materialcraft_formats::sbs::library::read(&format!("sbs://{}", recipe.path))?);
    }
    if key == "DoorFrame" {
        return Ok(br#"<svg xmlns="http://www.w3.org/2000/svg" width="512" height="512" viewBox="0 0 512 512"><path fill="white" fill-rule="evenodd" d="M88 480V220C88 38 424 38 424 220V480H370V220C370 109 142 109 142 220V480Z"/></svg>"#.to_vec());
    }
    // Every other recipe must explicitly select its source rather than invent replacement art.
    if key != "perm" {
        return Err(format!("no procedural generator for `{key}`").into());
    }
    let values: Vec<u16> = (0..16)
        .flat_map(|row| permutation(1024, 0x6d6174657269616c ^ (row * 313 + 1)).into_iter().map(|v| ((v as f32 / 1023.0) * 65535.0).round() as u16))
        .collect();
    let image = DynamicImage::ImageLuma16(ImageBuffer::from_raw(1024, 16, values).ok_or("invalid permutation image")?);
    let mut output = Cursor::new(Vec::new());
    image.write_to(&mut output, ImageFormat::Png)?;
    Ok(output.into_inner())
}

/// License and provenance files travel with the curated assets in converted libraries.
pub fn notices(key: &str) -> &'static [&'static str] {
    match key {
        "scratch_damage" | "leather" | "scratched_metal" | "moss01_bmp" => &["LICENSE_NICK_ROSE.txt"],
        "bluenoise_256" | "bluenoise_256_c" => &["LICENSE_BLUE_NOISE.txt"],
        "shuf4_64_32" | "sun" => &["LICENSE_HOSEK_WILKIE.txt", "sun_lookup_readme.txt"],
        "courier_new" => &["OFL-LiberationMono.txt"],
        _ => &[],
    }
}

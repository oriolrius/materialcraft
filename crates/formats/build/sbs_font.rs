//! Select just the SBS fallback face from the optional craft-fonts Latin catalog.
use sha2::{Digest, Sha256};
use std::{
    io::Read,
    path::{Component, Path, PathBuf},
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn read(path: &Path, limit: u64) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    std::fs::File::open(path)?.take(limit + 1).read_to_end(&mut bytes)?;
    if bytes.is_empty() || bytes.len() as u64 > limit {
        return Err(format!("{} is empty or exceeds its size limit", path.display()).into());
    }
    Ok(bytes)
}

fn checkout_file(root: &Path, relative: &str) -> Result<PathBuf> {
    // A manifest must not embed arbitrary files from the build host, including through symlinks.
    if relative.is_empty() || relative.contains(['\\', ':']) || !Path::new(relative).components().all(|part| matches!(part, Component::Normal(_))) {
        return Err("font manifest path is not checkout-relative".into());
    }
    let path = root.join(relative).canonicalize()?;
    if !path.starts_with(root) {
        return Err("font manifest path escapes the checkout".into());
    }
    println!("cargo:rerun-if-changed={}", path.display());
    Ok(path)
}

fn select(directory: &Path) -> Result<Vec<(PathBuf, PathBuf)>> {
    let root = directory.canonicalize()?;
    let manifest = checkout_file(&root, "fonts/latin-manifest.txt")?;
    let bytes = read(&manifest, 1 << 20)?;
    let entries = craft_fonts_build::parse_manifest(std::str::from_utf8(&bytes)?)?;
    let face = entries
        .iter()
        .find(|face| face.family == "Liberation Mono" && face.style == "Regular")
        .ok_or("Liberation Mono Regular is missing from the Latin font catalog")?;
    if face.licence != "OFL-1.1" || face.index != 0 {
        return Err("the SBS fallback must be the OFL-1.1 Liberation Mono Regular single font".into());
    }
    let font = checkout_file(&root, &face.file)?;
    let bytes = read(&font, 16 << 20)?;
    if !bytes.starts_with(b"\0\x01\0\0") || format!("{:x}", Sha256::digest(&bytes)) != face.sha256 {
        return Err("Liberation Mono Regular does not match its catalog checksum or TrueType format".into());
    }
    let license = checkout_file(&root, &face.licence_file)?;
    read(&license, 64 << 10)?;
    Ok(vec![("resources/LiberationMono-Regular.ttf".into(), font), ("resources/OFL-LiberationMono.txt".into(), license)])
}

pub fn assets() -> Result<Vec<(PathBuf, PathBuf)>> {
    println!("cargo:rerun-if-env-changed=CRAFT_FONTS_DIR");
    println!("cargo:rerun-if-env-changed=CRAFT_FONTS_REQUIRED");
    let required = std::env::var_os("CRAFT_FONTS_REQUIRED").is_some_and(|value| value == "1");
    let directory = std::env::var_os("CRAFT_FONTS_DIR").filter(|value| !value.is_empty());
    let selected = match directory {
        Some(directory) => select(Path::new(&directory)),
        None if required => Err("CRAFT_FONTS_REQUIRED=1 requires CRAFT_FONTS_DIR for the SBS fallback font".into()),
        None => return Ok(Vec::new()),
    };
    match selected {
        Ok(assets) => Ok(assets),
        Err(error) if required => Err(error),
        Err(error) => {
            println!("cargo:warning=building without the SBS fallback font from craft-fonts: {error}");
            Ok(Vec::new())
        }
    }
}

//! Bundle the same SBS library and original resources used by native disk installations.
#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn scan(root: &Path, folder: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    if folder.components().count() > 6 || files.len() > 4096 {
        return Err("bundled library exceeds its scan limits".into());
    }
    let path = root.join(folder);
    println!("cargo:rerun-if-changed={}", path.display());
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_str().ok_or("non-UTF-8 library filename")?;
        if name.starts_with('.') {
            continue;
        }
        let kind = entry.file_type()?;
        let relative = folder.join(name);
        if kind.is_dir() {
            scan(root, &relative, files)?;
        } else if kind.is_file() && (name.ends_with(".sbs") || relative.starts_with("resources")) {
            if files.len() >= 4096 {
                return Err("bundled library exceeds its file limit".into());
            }
            files.push(relative);
        } else if kind.is_symlink() {
            return Err("symlinks are not bundled library assets".into());
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let root = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").ok_or("missing manifest directory")?).join("../../assets/sbs");
    let out = PathBuf::from(std::env::var_os("OUT_DIR").ok_or("missing build output directory")?);
    let compressed = out.join("sbs-assets");
    std::fs::create_dir_all(&compressed)?;
    let mut files = Vec::new();
    scan(&root, Path::new(""), &mut files)?;
    files.sort_by_key(|path| path.to_string_lossy().replace('\\', "/"));
    let mut assets = String::from("const EMBEDDED_ASSETS: &[(&str, &[u8], usize)] = &[\n");
    let mut packages = String::from("pub const BUNDLED: &[&str] = &[\n");
    for (index, relative) in files.iter().enumerate() {
        let file = root.join(relative);
        let mut bytes = Vec::new();
        std::fs::File::open(&file)?.take((256 << 20) + 1).read_to_end(&mut bytes)?;
        if bytes.len() > 256 << 20 {
            return Err("bundled asset exceeds 256 MiB".into());
        }
        let mut encoder = flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::best());
        encoder.write_all(&bytes)?;
        std::fs::write(compressed.join(format!("{index}.deflate")), encoder.finish()?)?;
        let name = relative.to_str().ok_or("non-UTF-8 asset path")?.replace('\\', "/");
        assets.push_str(&format!("({name:?}, include_bytes!(concat!(env!(\"OUT_DIR\"), \"/sbs-assets/{index}.deflate\")), {}),\n", bytes.len()));
        if name.ends_with(".sbs") {
            packages.push_str(&format!("{:?},\n", format!("sbs://{name}")));
        }
    }
    assets.push_str("];\n");
    packages.push_str("];\n");
    std::fs::write(out.join("sbs_assets.rs"), format!("{assets}{packages}"))?;
    Ok(())
}

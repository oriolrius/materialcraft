//! Local SBS conversion: native graph implementations are retained, not independently reimplemented.
//! Presentation edits do not establish rights to redistribute the supplied graph implementations.
#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

mod convert;
mod font;
mod layout;
mod references;
mod resources;
#[cfg(test)]
mod tests;
mod xml;

use std::{collections::BTreeMap, io::Read, path::Path};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const MAX_FILE_BYTES: u64 = 64 << 20;

fn read(path: &Path) -> Result<Vec<u8>> {
    let metadata = std::fs::symlink_metadata(path)?;
    if !metadata.is_file() || metadata.len() > MAX_FILE_BYTES {
        return Err(format!("{} is not a regular file within the 64 MiB limit", path.display()).into());
    }
    let mut bytes = Vec::new();
    std::fs::File::open(path)?.take(MAX_FILE_BYTES + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_FILE_BYTES {
        return Err("file grew beyond the read limit".into());
    }
    Ok(bytes)
}

fn inventory(root: &Path, relative: &Path, files: &mut Vec<String>) -> Result<()> {
    if relative.components().count() > 6 || files.len() > 2048 {
        return Err("package inventory exceeds its depth or file limit".into());
    }
    let mut entries = std::fs::read_dir(root.join(relative))?.collect::<std::io::Result<Vec<_>>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let name = entry.file_name().into_string().map_err(|_| "non-UTF-8 package filename")?;
        let kind = entry.file_type()?;
        // Resource directories are deliberately never opened: replacements have independent recipes.
        if name.eq_ignore_ascii_case("resources") || name.ends_with(".resources") || name.starts_with('.') {
            continue;
        }
        let path = relative.join(&name);
        if kind.is_dir() {
            inventory(root, &path, files)?;
        } else if kind.is_file() && name.ends_with(".sbs") {
            let path = path.to_string_lossy().replace('\\', "/");
            materialcraft_formats::sbs::library::relative_path(&format!("sbs://{path}"))?;
            files.push(path);
        } else if kind.is_symlink() {
            return Err(format!("symlink refused in package inventory: {}", path.display()).into());
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let [source, destination, report] = args.as_slice() else {
        return Err("usage: sbs_library_build <source-directory> <destination-directory> <report.json>".into());
    };
    let source = Path::new(source).canonicalize()?;
    let destination = Path::new(destination).canonicalize()?;
    if source == destination || destination.starts_with(&source) || source.starts_with(&destination) {
        return Err("source and destination trees must be separate".into());
    }
    let baseline = read(&destination.join("functions.sbs"))?;
    let baseline_text = std::str::from_utf8(&baseline)?;
    let baseline_doc = xml::parse(baseline_text)?;
    let standard_functions = references::function_paths(&baseline_doc);
    let baseline_uid = xml::value(baseline_doc.root_element(), "fileUID").unwrap_or("0");
    let mut names = Vec::new();
    inventory(&source, Path::new(""), &mut names)?;
    names.sort();
    let mut identities: BTreeMap<_, _> = names.iter().map(|name| (name.clone(), xml::identity(name))).collect();
    identities.insert("functions.sbs".into(), baseline_uid.into());
    let mut outputs = BTreeMap::new();
    let mut reports = Vec::new();
    let mut needed = std::collections::BTreeSet::new();
    let mut total = 0_usize;
    for name in names.iter().filter(|name| name.as_str() != "functions.sbs") {
        let bytes = read(&source.join(name))?;
        let (source, qualified) = references::qualify(std::str::from_utf8(&bytes)?, &standard_functions)?;
        let mut converted = convert::package(&source, name, &identities)?;
        converted.report["qualified_standard_function_references"] = serde_json::json!(qualified);
        total = total.checked_add(converted.xml.len()).ok_or("library size overflow")?;
        if total > 512 << 20 {
            return Err("converted library exceeds 512 MiB".into());
        }
        needed.extend(converted.resources.iter().copied());
        reports.push(converted.report);
        outputs.insert(name.clone(), converted.xml.into_bytes());
    }
    let mut assets = Vec::new();
    for key in needed {
        let recipe = resources::recipe(key)?;
        let bytes = resources::generate(key)?;
        assets.push(serde_json::json!({"path": recipe.path, "recipe": recipe.description, "bytes": bytes.len()}));
        outputs.insert(recipe.path.into(), bytes);
    }
    // Validate the complete dependency closure before writing any package or asset.
    let available = outputs.keys().cloned().chain(std::iter::once("functions.sbs".into())).collect();
    for (name, bytes) in outputs.iter().filter(|(name, _)| name.ends_with(".sbs")) {
        convert::verify_references(std::str::from_utf8(bytes)?, name, &available)?;
    }
    for (name, bytes) in &outputs {
        let target = destination.join(name);
        if target.exists() && read(&target)? != *bytes {
            return Err(format!("refusing to overwrite different existing content: {}", target.display()).into());
        }
        if let Some(parent) = target.parent() {
            // Refuse existing symlink parents before creating output directories.
            let mut ancestor = parent;
            while ancestor != destination {
                if ancestor.exists() && !ancestor.canonicalize()?.starts_with(&destination) {
                    return Err("output directory escapes the destination".into());
                }
                ancestor = ancestor.parent().ok_or("invalid output directory")?;
            }
        }
    }
    for (name, bytes) in &outputs {
        let target = destination.join(name);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }
        materialcraft_formats::write_atomic(&target, bytes)?;
    }
    if read(&destination.join("functions.sbs"))? != baseline {
        return Err("the existing function library changed during conversion".into());
    }
    let report_data = serde_json::json!({
        "version": 1, "converted_packages": reports.len(), "preserved_packages": ["functions.sbs"],
        "graph_implementations": "retained from the supplied packages; presentation changes do not establish redistribution rights",
        "validation": "native semantic projection and dependency closure checked; external runtime parity unverified",
        "resources": assets, "packages": reports,
    });
    materialcraft_formats::write_atomic(Path::new(report), &serde_json::to_vec_pretty(&report_data)?)?;
    eprintln!("Converted {} packages and generated {} resources. Existing functions.sbs unchanged.", reports.len(), assets.len());
    Ok(())
}

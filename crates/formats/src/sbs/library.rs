//! Resolve the portable SBS library alias against the application's asset directory.

include!(concat!(env!("OUT_DIR"), "/sbs_assets.rs"));

/// Validate alias paths before joining a filesystem root. No package may escape the library.
pub fn relative_path(uri: &str) -> Result<&str, String> {
    let path = uri.strip_prefix("sbs://").ok_or("not an SBS library alias")?;
    if path.is_empty()
        || path.len() > 4096
        || path.contains(['\\', ':', '?', '#', '%', '\0'])
        || path.split('/').any(|part| part.is_empty() || matches!(part, "." | ".."))
    {
        return Err("invalid SBS library path".into());
    }

    Ok(path)
}

/// Resolve a package-relative resource or dependency without leaving the library root.
pub fn resolve(owner: &str, reference: &str) -> Result<String, String> {
    let owner = relative_path(owner)?;
    if reference.starts_with("sbs://") {
        relative_path(reference)?;
        return Ok(reference.into());
    }

    let reference = reference.trim().replace('\\', "/");
    if reference.is_empty() || reference.len() > 4096 || reference.starts_with('/') || reference.contains([':', '?', '#', '%', '\0']) {
        return Err("invalid relative SBS library path".into());
    }

    let mut parts: Vec<_> = owner.split('/').collect();
    parts.pop();
    for part in reference.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                if parts.pop().is_none() {
                    return Err("SBS reference escapes its library directory".into());
                }
            }
            part => parts.push(part),
        }
    }

    let uri = format!("sbs://{}", parts.join("/"));
    relative_path(&uri)?;
    Ok(uri)
}

/// The native asset root may be configured for installations with a separate data directory.
#[cfg(not(target_arch = "wasm32"))]
pub fn root() -> std::path::PathBuf {
    if let Some(root) = std::env::var_os("MATERIALCRAFT_SBS_DIR") {
        return root.into();
    }

    if let Ok(executable) = std::env::current_exe()
        && let Some(parent) = executable.parent()
        && parent.join("assets/sbs").is_dir()
    {
        return parent.join("assets/sbs");
    }

    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/sbs")
}

/// A canonical library location, including a new filename beneath an existing library folder.
#[cfg(not(target_arch = "wasm32"))]
pub fn contains_file(path: &std::path::Path) -> bool {
    let Ok(root) = root().canonicalize() else { return false };
    if let Ok(file) = path.canonicalize() {
        return file.starts_with(root);
    }
    path.parent().and_then(|parent| parent.canonicalize().ok()).is_some_and(|parent| parent.starts_with(root))
}

/// Read a validated library alias from the configured directory or the embedded fallback.
pub fn read(uri: &str) -> Result<Vec<u8>, String> {
    let relative = relative_path(uri)?;
    #[cfg(not(target_arch = "wasm32"))]
    {
        let root = root();
        if root.exists() || std::env::var_os("MATERIALCRAFT_SBS_DIR").is_some() {
            let root = root.canonicalize().map_err(|error| error.to_string())?;
            let file = root.join(relative).canonicalize().map_err(|error| format!("Missing library asset {uri}: {error}"))?;
            if !file.starts_with(&root) {
                return Err("SBS library symlink escapes its asset directory".into());
            }

            return std::fs::read(file).map_err(|error| error.to_string());
        }
    }

    read_embedded(relative)
}

/// Compressed assets keep installed and browser builds independent of the source checkout.
fn read_embedded(relative: &str) -> Result<Vec<u8>, String> {
    use std::io::Read;
    let index = EMBEDDED_ASSETS.binary_search_by_key(&relative, |(name, _, _)| *name).map_err(|_| format!("Missing library asset sbs://{relative}"))?;
    let (_, compressed, size) = EMBEDDED_ASSETS.get(index).ok_or("invalid bundled asset index")?;
    if *size > 256 << 20 {
        return Err("bundled asset exceeds 256 MiB".into());
    }
    let mut bytes = Vec::new();
    flate2::read::DeflateDecoder::new(*compressed).take(*size as u64 + 1).read_to_end(&mut bytes).map_err(|error| error.to_string())?;
    if bytes.len() != *size {
        return Err("bundled asset has an invalid decompressed length".into());
    }
    Ok(bytes)
}

/// Bounds for scanning the library folder: how deep, and how many packages
const MAX_DEPTH: usize = 6;
const MAX_FILES: usize = 2048;

/// Every SBS package in the library as `sbs://` paths, sorted: the files under the asset folder
/// (resource folders skipped), or the bundled packages when there is no folder
pub fn files() -> Vec<String> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let root = root();
        if root.is_dir() {
            let mut found = Vec::new();
            scan(&root, "", 0, &mut found);
            found.sort();
            return found;
        }
    }
    BUNDLED.iter().map(|uri| uri.to_string()).collect()
}

/// Discover packages in deterministic order within the depth and file limits.
#[cfg(not(target_arch = "wasm32"))]
fn scan(directory: &std::path::Path, prefix: &str, depth: usize, found: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(directory) else { return };
    let mut entries: Vec<_> = entries.flatten().collect();
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        if found.len() >= MAX_FILES {
            return;
        }

        let Some(name) = entry.file_name().to_str().map(str::to_string) else { continue };
        let Ok(kind) = entry.file_type() else { continue };
        let uri = format!("sbs://{prefix}{name}");

        // Symlinks are skipped: `read` refuses files that leave the asset folder anyway
        if kind.is_dir() && depth < MAX_DEPTH && !name.ends_with(".resources") && !name.starts_with('.') {
            scan(&entry.path(), &format!("{prefix}{name}/"), depth + 1, found);
        } else if kind.is_file() && name.to_ascii_lowercase().ends_with(".sbs") && relative_path(&uri).is_ok() {
            found.push(uri);
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    /// The shipped fallback preserves the exact SBS and resource bytes.
    #[test]
    fn embedded_library_contains_every_disk_asset() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/sbs");
        for (relative, _, _) in EMBEDDED_ASSETS {
            assert_eq!(read_embedded(relative).unwrap(), std::fs::read(root.join(relative)).unwrap(), "{relative}");
        }
        assert!(read_embedded("resources/cursor.otf").unwrap().starts_with(b"OTTO"));
    }

    /// Discovered packages have unique valid aliases and the embedded fallback is present.
    #[test]
    fn the_library_lists_every_package_once_and_bundles_a_fallback() {
        let files = files();
        for uri in BUNDLED {
            assert!(files.iter().any(|file| file == uri), "{uri} in {files:?}");
            assert!(!read(uri).unwrap().is_empty());
        }
        let mut unique = files.clone();
        unique.dedup();
        assert_eq!(unique, files);
        assert!(files.iter().all(|file| relative_path(file).is_ok() && file.ends_with(".sbs")));
    }
}

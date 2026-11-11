#[cfg(not(target_arch = "wasm32"))]
use crate::{FormatError, Result};

/// Longest file name `safe_file_name` returns, in characters
const MAX_NAME: usize = 100;

/// Longest extension kept when a long name is cut
const MAX_EXT: usize = 16;

/// Cut a name over `MAX_NAME` characters in its stem, keeping the last `.ext`; a hash of the full name keeps two long names with one prefix apart
fn shorten(name: String) -> String {
    if name.chars().count() <= MAX_NAME {
        return name;
    }

    let hash = name.bytes().fold(0xcbf29ce484222325u64, |h, b| (h ^ u64::from(b)).wrapping_mul(0x100000001b3));
    let (stem, ext) = match name.rsplit_once('.') {
        Some((stem, ext)) if !stem.is_empty() && ext.chars().count() <= MAX_EXT => (stem, format!(".{ext}")),
        _ => (name.as_str(), String::new()),
    };
    let keep = MAX_NAME - ext.chars().count() - 9;
    format!("{}-{:08x}{ext}", stem.chars().take(keep).collect::<String>(), hash as u32)
}

pub fn safe_file_name(name: &str) -> String {
    let base = name.rsplit(['/', '\\']).next().unwrap_or(name);
    let cleaned: String = base.chars().map(|c| if c.is_alphanumeric() || "._- ".contains(c) { c } else { '_' }).collect();
    // Windows drops trailing dots and spaces, so `CON.` would still open the device
    let cleaned = cleaned.trim_start_matches('.').trim_end_matches(['.', ' ']).to_string();
    let cleaned = shorten(cleaned);
    if cleaned.is_empty() {
        return "resource".into();
    }

    let stem = cleaned.split('.').next().unwrap_or("").trim_end().to_ascii_uppercase();
    let mut chars = stem.chars();
    let device = matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL" | "CONIN$" | "CONOUT$");
    // COM1..COM9 and LPT1..LPT9, also with the superscript digits Windows treats as digits
    let numbered = stem.chars().count() == 4
        && (stem.starts_with("COM") || stem.starts_with("LPT"))
        && chars.next_back().is_some_and(|c| matches!(c, '1'..='9' | '\u{b9}' | '\u{b2}' | '\u{b3}'));
    let reserved = device || numbered;
    if reserved { format!("_{cleaned}") } else { cleaned }
}

/// Read a whole regular file of at most `max` bytes: a FIFO, a device or an oversized file is refused before anything is allocated
#[cfg(not(target_arch = "wasm32"))]
pub fn read_capped(path: impl AsRef<std::path::Path>, max: u64) -> Result<Vec<u8>> {
    use std::io::Read;
    let path = path.as_ref();
    let io = |e: std::io::Error| FormatError::Io(format!("{}: {e}", path.display()));
    // Check before open: opening a FIFO for reading blocks until a writer appears
    let regular = |m: std::fs::Metadata| if m.is_file() { Ok(m.len()) } else { Err(FormatError::Io(format!("{}: not a regular file", path.display()))) };
    regular(std::fs::metadata(path).map_err(io)?)?;
    let file = std::fs::File::open(path).map_err(io)?;
    let len = regular(file.metadata().map_err(io)?)?;
    if len > max {
        return Err(FormatError::Io(format!("{}: file is larger than {max} bytes", path.display())));
    }

    let mut bytes = Vec::with_capacity(len as usize);
    file.take(max.saturating_add(1)).read_to_end(&mut bytes).map_err(io)?;
    if bytes.len() as u64 > max {
        return Err(FormatError::Io(format!("{}: file is larger than {max} bytes", path.display())));
    }
    Ok(bytes)
}

/// The file stem of an exported output: `pattern` with `$(graph)`, `$(identifier)` and `$(usage)` replaced, sanitized for file systems
pub fn output_name(pattern: &str, graph: &str, identifier: &str, usage: &str) -> String {
    let s = pattern.replace("$(graph)", graph).replace("$(identifier)", identifier).replace("$(usage)", usage);
    safe_file_name(&s)
}

/// Write `bytes` to `path` through a temporary file and a rename, so a failed write never leaves a truncated file behind
#[cfg(not(target_arch = "wasm32"))]
pub fn write_atomic(path: &std::path::Path, bytes: &[u8]) -> Result<()> {
    let tmp = path.with_extension(format!("{}.tmp", path.extension().and_then(|e| e.to_str()).unwrap_or("out")));
    std::fs::write(&tmp, bytes).map_err(|e| FormatError::Io(format!("{}: {e}", tmp.display())))?;
    std::fs::rename(&tmp, path).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        FormatError::Io(format!("{}: {e}", path.display()))
    })
}

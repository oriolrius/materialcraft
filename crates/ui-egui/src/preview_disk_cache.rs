//! Disposable PNG thumbnails. All filesystem work belongs to the existing preview workers.

use std::fs::{self, File};
use std::io::{self, Cursor, Read};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, SystemTime};

use image::{ImageDecoder, ImageEncoder};
use sha2::{Digest, Sha256};

const MAX_ENTRIES: usize = 4096;
const MAX_FILE_BYTES: u64 = 128 * 1024;
const MAX_SIDE: u32 = 128;

pub(super) struct PreviewDiskCache {
    root: PathBuf,
    renderer: [u8; 32],
    budget: u64,
    entries: usize,
}

impl PreviewDiskCache {
    pub fn configured() -> Option<Self> {
        // Scripted runs must not touch the user's cache. An explicit override lets tests and
        // agents exercise persistence in an isolated folder; an empty override disables it.
        let root = match std::env::var_os("MATERIALCRAFT_CACHE_DIR") {
            Some(path) if path.is_empty() => return None,
            Some(path) => PathBuf::from(path),
            None if cfg!(test) || std::env::var_os("MATERIALCRAFT_NO_PREFS").is_some() => return None,
            // The existing shared path resolver follows the suite's cache-folder policy,
            // including redirected Windows profiles and absolute XDG paths.
            None => craft_update::Paths::per_user("materialcraft").ok()?.cache,
        };
        Self::open(root.join("previews"), 64 * 1024 * 1024)
    }

    /// Called on a worker. The executable digest also invalidates development builds that keep
    /// the same version number. It is streamed once per process, never on the event-loop thread.
    pub fn open(root: PathBuf, budget: u64) -> Option<Self> {
        static RENDERER: OnceLock<Option<[u8; 32]>> = OnceLock::new();
        let renderer = *RENDERER.get_or_init(|| {
            let mut file = File::open(std::env::current_exe().ok()?).ok()?.take((1 << 30) + 1);
            let mut hash = Sha256::new();
            let bytes = io::copy(&mut file, &mut hash).ok()?;
            (bytes <= 1 << 30).then(|| hash.finalize().into())
        });
        Some(Self { root, renderer: renderer?, budget, entries: MAX_ENTRIES })
    }

    pub fn get_or_render(&self, key: [u8; 32], side: u32, cancel: &AtomicBool, render: impl FnOnce() -> Option<Vec<u8>>) -> Option<Vec<u8>> {
        if cancel.load(Ordering::Relaxed) {
            return None;
        }
        let path = self.path(key, side);
        if let Some(pixels) = self.read(&path, side) {
            return (!cancel.load(Ordering::Relaxed)).then_some(pixels);
        }
        let pixels = render()?;
        if cancel.load(Ordering::Relaxed) {
            return None;
        }
        // A cache is optional: unavailable storage must not hide a successfully rendered tile.
        if let Err(error) = self.write(&path, side, &pixels) {
            log::debug!("Library preview persistence skipped: {error}");
        }
        Some(pixels)
    }

    fn path(&self, key: [u8; 32], side: u32) -> PathBuf {
        let mut hash = Sha256::new();
        hash.update(b"materialcraft-preview-png-v1");
        hash.update(self.renderer);
        hash.update(key);
        hash.update(side.to_le_bytes());
        self.root.join(format!("{:x}.png", hash.finalize()))
    }

    fn read(&self, path: &Path, side: u32) -> Option<Vec<u8>> {
        if side == 0 || side > MAX_SIDE || self.budget == 0 {
            return None;
        }
        let metadata = fs::symlink_metadata(path).ok()?;
        if !metadata.is_file() || metadata.len() > MAX_FILE_BYTES {
            return None;
        }
        let mut file = File::open(path).ok()?;
        let mut bytes = Vec::new();
        Read::by_ref(&mut file).take(MAX_FILE_BYTES + 1).read_to_end(&mut bytes).ok()?;
        if bytes.len() as u64 > MAX_FILE_BYTES {
            return None;
        }
        let mut limits = image::Limits::default();
        limits.max_image_width = Some(MAX_SIDE);
        limits.max_image_height = Some(MAX_SIDE);
        limits.max_alloc = Some(1024 * 1024);
        let decoder = image::codecs::png::PngDecoder::with_limits(Cursor::new(bytes), limits).ok()?;
        if decoder.dimensions() != (side, side) || decoder.color_type() != image::ColorType::Rgba8 {
            return None;
        }
        let mut pixels = vec![0; (side * side * 4) as usize];
        decoder.read_image(&mut pixels).ok()?;
        // Approximate LRU across launches, without a metadata write on every tile lookup.
        if metadata.modified().ok()?.elapsed().is_ok_and(|age| age >= Duration::from_secs(3600)) {
            let _ = file.set_modified(SystemTime::now());
        }
        Some(pixels)
    }

    fn write(&self, path: &Path, side: u32, pixels: &[u8]) -> io::Result<()> {
        if side == 0 || side > MAX_SIDE || pixels.len() != (side * side * 4) as usize || self.budget == 0 {
            return Ok(());
        }
        let mut bytes = Vec::new();
        image::codecs::png::PngEncoder::new(&mut bytes).write_image(pixels, side, side, image::ExtendedColorType::Rgba8).map_err(io::Error::other)?;
        if bytes.len() as u64 > MAX_FILE_BYTES || bytes.len() as u64 > self.budget {
            return Ok(());
        }
        fs::create_dir_all(&self.root)?;
        // Coordinate the budget across windows/processes. A busy writer is a harmless cache
        // miss; do not block a preview worker waiting for another process to finish.
        let lock = File::options().read(true).write(true).create(true).truncate(false).open(self.root.join("cache.lock"))?;
        lock.try_lock().map_err(io::Error::other)?;
        // craft-atomic deliberately follows document symlinks. Cache entries are disposable
        // files owned by this cache, so never follow one into an unrelated document instead.
        if fs::symlink_metadata(path).is_ok_and(|metadata| !metadata.is_file()) {
            return Err(io::Error::other("preview cache entry is not a regular file"));
        }
        self.reserve(path, bytes.len() as u64)?;
        craft_atomic::WriteOptions::default().with_new_file_permissions(craft_atomic::NewFilePermissions::Private).atomic_write(path, &bytes)
    }

    fn reserve(&self, target: &Path, incoming: u64) -> io::Result<()> {
        let mut entries = Vec::new();
        let mut total = 0u64;
        // Bound both scan time and metadata allocations in an externally populated directory.
        for (index, entry) in fs::read_dir(&self.root)?.enumerate() {
            if index >= MAX_ENTRIES * 2 {
                return Err(io::Error::other("preview cache directory exceeds its entry limit"));
            }
            let entry = entry?;
            if !entry.file_type()?.is_file() {
                continue;
            }
            let name = entry.file_name();
            let name = name.to_string_lossy();
            // The cache lock excludes every live writer. Matching staging files can only be
            // leftovers from an interrupted atomic save, so reclaim them before growing again.
            if staging_name(&name) {
                fs::remove_file(entry.path())?;
                continue;
            }
            if !cache_name(&name) || entry.path() == target {
                continue;
            }
            let metadata = entry.metadata()?;
            total = total.saturating_add(metadata.len());
            entries.push((metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH), entry.path(), metadata.len()));
        }
        entries.sort_unstable_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
        let mut count = entries.len();
        for (_, path, size) in entries {
            if total.saturating_add(incoming) <= self.budget && count < self.entries {
                break;
            }
            fs::remove_file(path)?;
            total = total.saturating_sub(size);
            count = count.saturating_sub(1);
        }
        Ok(())
    }
}

fn cache_name(name: &str) -> bool {
    name.strip_suffix(".png").is_some_and(digest_name)
}

fn digest_name(stem: &str) -> bool {
    stem.len() == 64 && stem.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn staging_name(name: &str) -> bool {
    let Some((stem, suffix)) = name.strip_prefix('.').and_then(|name| name.strip_suffix(".craft_tmp")).and_then(|name| name.split_once('.')) else {
        return false;
    };
    digest_name(stem) && suffix.split_once('-').is_some_and(|(pid, counter)| pid.parse::<u32>().is_ok_and(|pid| pid > 0) && counter.parse::<u64>().is_ok())
}

#[cfg(test)]
#[path = "preview_disk_cache.test.rs"]
mod tests;

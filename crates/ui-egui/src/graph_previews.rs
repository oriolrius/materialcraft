use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, TryRecvError};

use egui::{ColorImage, TextureHandle, TextureOptions};
use materialcraft_engine::NodeLibrary;

use super::LIBRARY_PREVIEW_SIZE;

type Key = (String, String);
const MAX_TEXTURES: usize = 256; // About 5 MiB of RGBA pixels, independent of library size.
const MAX_VISIBLE_REQUESTS: usize = 64;

/// Only one cook may run; every frame replaces the pending work with its visible tiles.
#[derive(Default)]
pub(super) struct GraphPreviews {
    #[cfg(not(target_arch = "wasm32"))]
    pub persist: bool,
    library: Option<Arc<NodeLibrary>>,
    textures: HashMap<Key, (Option<TextureHandle>, u64)>,
    wanted: Vec<Key>,
    active: Option<Active>,
    frame: u64,
}

struct Active {
    library: Arc<NodeLibrary>,
    key: Key,
    cancel: Arc<AtomicBool>,
    result: Receiver<Option<Vec<u8>>>,
}

impl Drop for Active {
    fn drop(&mut self) {
        // Closing the app or its preview state must not leave background cooks running.
        self.cancel.store(true, Ordering::Relaxed);
    }
}

impl GraphPreviews {
    pub fn get(&mut self, _ctx: &egui::Context, library: &Arc<NodeLibrary>, package: &str, graph: &str) -> Option<egui::TextureId> {
        self.set_library(library);
        let key = (package.to_string(), graph.to_string());
        if let Some((texture, used)) = self.textures.get_mut(&key) {
            *used = self.frame;
            return texture.as_ref().map(TextureHandle::id);
        }
        if self.wanted.len() < MAX_VISIBLE_REQUESTS && !self.wanted.contains(&key) {
            self.wanted.push(key);
        }
        None
    }

    fn set_library(&mut self, library: &Arc<NodeLibrary>) {
        if !self.library.as_ref().is_some_and(|current| Arc::ptr_eq(current, library)) {
            self.library = Some(library.clone());
            self.textures.clear();
            self.wanted.clear();
            if let Some(active) = &self.active {
                active.cancel.store(true, Ordering::Relaxed);
            }
            // Keep the receiver until the old cook ends: repeated reloads cannot pile up workers.
        }
    }

    /// Called after drawing the panels, including frames where the Library is hidden.
    pub fn collect(&mut self, ctx: &egui::Context, library: &Arc<NodeLibrary>, foreground_busy: bool) {
        // A hidden panel must also release the previous library after a reload.
        self.set_library(library);
        self.frame = self.frame.saturating_add(1);
        let finished = self.active.as_ref().and_then(|active| match active.result.try_recv() {
            Ok(image) => Some(image),
            Err(TryRecvError::Disconnected) => Some(None),
            Err(TryRecvError::Empty) => None,
        });
        if let Some(image) = finished
            && let Some(active) = self.active.take()
        {
            if !active.cancel.load(Ordering::Relaxed) && self.library.as_ref().is_some_and(|library| Arc::ptr_eq(library, &active.library)) {
                let texture = image.map(|rgba| {
                    let image = ColorImage::from_rgba_unmultiplied([LIBRARY_PREVIEW_SIZE as usize; 2], &rgba);
                    ctx.load_texture(format!("library-graph-{}#{}", active.key.0, active.key.1), image, TextureOptions::LINEAR)
                });
                self.remember(active.key.clone(), texture);
            }
            ctx.request_repaint();
        }
        if let Some(active) = &self.active {
            if foreground_busy || !self.wanted.contains(&active.key) {
                active.cancel.store(true, Ordering::Relaxed);
            }
        } else if !foreground_busy
            && let Some(library) = self.library.clone()
            && let Some(key) = self.wanted.iter().rev().find(|key| !self.textures.contains_key(*key)).cloned()
        {
            self.start(ctx, library, key);
        }
        self.wanted.clear();
    }

    fn remember(&mut self, key: Key, texture: Option<TextureHandle>) {
        self.textures.insert(key, (texture, self.frame));
        while self.textures.len() > MAX_TEXTURES {
            let Some(oldest) = self.textures.iter().min_by_key(|(_, (_, used))| *used).map(|(key, _)| key.clone()) else { break };
            self.textures.remove(&oldest);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn start(&mut self, ctx: &egui::Context, library: Arc<NodeLibrary>, key: Key) {
        let (sender, result) = std::sync::mpsc::sync_channel(1);
        let cancel = Arc::new(AtomicBool::new(false));
        let (thread_cancel, thread_library, thread_key, context) = (cancel.clone(), library.clone(), key.clone(), ctx.clone());
        let persist = self.persist;
        let spawned = std::thread::Builder::new().name("materialcraft-library-graphs".into()).spawn(move || {
            let render = || {
                NodeLibrary::preview_cancellable(&thread_library, &thread_key.0, &thread_key.1, LIBRARY_PREVIEW_SIZE, &thread_cancel)
                    .map(|image| image.resize(LIBRARY_PREVIEW_SIZE, LIBRARY_PREVIEW_SIZE, true).to_rgba8_display())
            };
            let disk = persist.then(super::preview_disk_cache::PreviewDiskCache::configured).flatten();
            let fingerprint = disk.as_ref().and_then(|_| thread_library.preview_fingerprint(&thread_key.0, &thread_key.1, &thread_cancel));
            let rgba = match (disk, fingerprint) {
                (Some(disk), Some(key)) => disk.get_or_render(key, LIBRARY_PREVIEW_SIZE, &thread_cancel, render),
                _ => render(),
            };
            let _ = sender.send(rgba);
            context.request_repaint();
        });
        if spawned.is_ok() {
            self.active = Some(Active { library, key, cancel, result });
        } else {
            self.remember(key, None);
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn start(&mut self, _ctx: &egui::Context, _library: Arc<NodeLibrary>, key: Key) {
        // The browser has no cook worker; keep the category icon without retrying every frame.
        self.remember(key, None);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn active(library: Arc<NodeLibrary>, key: Key) -> (Active, std::sync::mpsc::Sender<Option<Vec<u8>>>) {
        let (sender, result) = std::sync::mpsc::channel();
        (Active { library, key, cancel: Arc::new(AtomicBool::new(false)), result }, sender)
    }

    #[test]
    fn hidden_tiles_cancel_the_running_job_and_discard_pending_requests() {
        let context = egui::Context::default();
        let library = Arc::new(NodeLibrary::default());
        let mut previews = GraphPreviews::default();
        let key = ("test".into(), "graph".into());
        let (job, _sender) = active(library.clone(), key.clone());
        let cancelled = job.cancel.clone();
        previews.active = Some(job);
        previews.library = Some(library.clone());
        for graph in 0..1_000 {
            previews.get(&context, &library, "test", &graph.to_string());
        }
        assert_eq!(previews.wanted.len(), MAX_VISIBLE_REQUESTS);
        previews.collect(&context, &library, false);
        assert!(cancelled.load(Ordering::Relaxed));
        assert!(previews.wanted.is_empty());
        assert_eq!(previews.active.as_ref().unwrap().key, key, "wait for cancellation before starting another worker");
    }

    #[test]
    fn reload_waits_for_the_previous_worker_and_releases_its_library() {
        let context = egui::Context::default();
        let library = Arc::new(NodeLibrary::default());
        let old = Arc::downgrade(&library);
        let (job, sender) = active(library.clone(), ("test".into(), "graph".into()));
        let mut previews = GraphPreviews { library: Some(library), active: Some(job), ..Default::default() };
        let new_library = Arc::new(NodeLibrary::default());
        previews.get(&context, &new_library, "new", "graph");
        assert!(previews.active.as_ref().unwrap().cancel.load(Ordering::Relaxed));
        previews.wanted.clear();
        sender.send(None).unwrap();
        previews.collect(&context, &new_library, false);
        assert!(previews.active.is_none());
        assert!(previews.textures.is_empty(), "a cancelled result must be eligible for another request");
        assert!(old.upgrade().is_none());
    }

    #[test]
    fn preview_cache_releases_old_gpu_textures_and_keeps_recent_entries() {
        let context = egui::Context::default();
        let mut previews = GraphPreviews::default();
        let texture = context.load_texture("old", ColorImage::new([1, 1], vec![egui::Color32::WHITE]), TextureOptions::LINEAR);
        let old_id = texture.id();
        previews.remember(("test".into(), "old".into()), Some(texture));
        for index in 0..MAX_TEXTURES {
            previews.frame += 1;
            previews.remember(("test".into(), index.to_string()), None);
        }
        assert_eq!(previews.textures.len(), MAX_TEXTURES);
        assert!(context.tex_manager().read().meta(old_id).is_none(), "eviction must drop the GPU texture handle");
    }

    #[test]
    fn editing_takes_priority_and_dropping_preview_state_cancels_work() {
        let context = egui::Context::default();
        let library = Arc::new(NodeLibrary::default());
        let mut previews = GraphPreviews::default();
        previews.get(&context, &library, "test", "graph");
        previews.collect(&context, &library, true);
        assert!(previews.active.is_none());
        let (job, _sender) = active(library.clone(), ("test".into(), "graph".into()));
        let cancelled = job.cancel.clone();
        previews.active = Some(job);
        drop(previews);
        assert!(cancelled.load(Ordering::Relaxed));
    }
}

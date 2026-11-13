use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, TryRecvError};

use egui::{ColorImage, TextureHandle, TextureOptions};
use materialcraft_engine::NodeLibrary;

use super::LIBRARY_PREVIEW_SIZE;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum Key {
    Graph(String, String),
    Native(String),
}
const MAX_TEXTURES: usize = 256; // About 5 MiB of RGBA pixels, independent of library size.
const MAX_VISIBLE_REQUESTS: usize = 64;

/// Only one cook may run; every frame replaces the pending work with its visible tiles.
#[derive(Default)]
pub(super) struct GraphPreviews {
    #[cfg(not(target_arch = "wasm32"))]
    pub persistence: Option<super::preview_cache_settings::Settings>,
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
        self.request(Key::Graph(package.to_string(), graph.to_string()))
    }

    pub fn native(&mut self, _ctx: &egui::Context, id: &str) -> Option<egui::TextureId> {
        self.request(Key::Native(id.into()))
    }

    fn request(&mut self, key: Key) -> Option<egui::TextureId> {
        if let Some((texture, used)) = self.textures.get_mut(&key) {
            *used = self.frame;
            return texture.as_ref().map(TextureHandle::id);
        }
        if self.wanted.len() < MAX_VISIBLE_REQUESTS && !self.wanted.contains(&key) {
            self.wanted.push(key);
        }
        None
    }

    pub fn clear(&mut self) {
        self.textures.clear();
        self.wanted.clear();
        if let Some(active) = &self.active {
            active.cancel.store(true, Ordering::Relaxed);
        }
        // Keep the receiver until its worker exits so clearing cannot pile up render threads.
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
                    ctx.load_texture(format!("library-preview-{:?}", active.key), image, TextureOptions::LINEAR)
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
            && self.cache_ready()
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
        let persistence = self.persistence.clone();
        let spawned = std::thread::Builder::new().name("materialcraft-library-graphs".into()).spawn(move || {
            let render = || {
                if thread_cancel.load(Ordering::Relaxed) {
                    return None;
                }
                match &thread_key {
                    Key::Native(id) => super::render_native(id),
                    Key::Graph(package, graph) => NodeLibrary::preview_cancellable(&thread_library, package, graph, LIBRARY_PREVIEW_SIZE, &thread_cancel)
                        .map(|image| image.resize(LIBRARY_PREVIEW_SIZE, LIBRARY_PREVIEW_SIZE, true).to_rgba8_display()),
                }
            };
            let disk = persistence.and_then(super::preview_disk_cache::PreviewDiskCache::configured);
            let fingerprint = disk.as_ref().filter(|disk| disk.enabled()).and_then(|_| match &thread_key {
                Key::Native(id) => {
                    use sha2::Digest;
                    Some(sha2::Sha256::digest(format!("materialcraft-atomic-preview-v1:{id}")).into())
                }
                Key::Graph(package, graph) => thread_library.preview_fingerprint(package, graph, &thread_cancel),
            });
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

    fn cache_ready(&self) -> bool {
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(settings) = &self.persistence {
            return !settings.get().clear_pending;
        }
        true
    }

    #[cfg(target_arch = "wasm32")]
    fn start(&mut self, ctx: &egui::Context, _library: Arc<NodeLibrary>, key: Key) {
        // Native previews remain available in the browser without a filesystem or cook worker.
        let rgba = match &key {
            Key::Native(id) => super::render_native(id),
            Key::Graph(_, _) => None,
        };
        let texture = rgba.map(|rgba| {
            ctx.load_texture(
                format!("library-preview-{key:?}"),
                ColorImage::from_rgba_unmultiplied([LIBRARY_PREVIEW_SIZE as usize; 2], &rgba),
                TextureOptions::LINEAR,
            )
        });
        self.remember(key, texture);
        ctx.request_repaint();
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
        let key = Key::Graph("test".into(), "graph".into());
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
        let (job, sender) = active(library.clone(), Key::Graph("test".into(), "graph".into()));
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
        previews.remember(Key::Graph("test".into(), "old".into()), Some(texture));
        for index in 0..MAX_TEXTURES {
            previews.frame += 1;
            previews.remember(Key::Graph("test".into(), index.to_string()), None);
        }
        assert_eq!(previews.textures.len(), MAX_TEXTURES);
        assert!(context.tex_manager().read().meta(old_id).is_none(), "eviction must drop the GPU texture handle");
    }

    #[test]
    fn clear_discards_loaded_and_pending_tiles_and_rebuilds_only_requested_tiles() {
        let context = egui::Context::default();
        let library = Arc::new(NodeLibrary::default());
        let mut previews = GraphPreviews::default();
        previews.set_library(&library);
        let key = Key::Native("blur".into());
        let texture = context.load_texture("old", ColorImage::new([1, 1], vec![egui::Color32::WHITE]), TextureOptions::LINEAR);
        let old_id = texture.id();
        previews.remember(key.clone(), Some(texture));
        let (job, sender) = active(library.clone(), key);
        let cancelled = job.cancel.clone();
        previews.active = Some(job);
        previews.native(&context, "levels");
        previews.clear();
        assert!(previews.textures.is_empty());
        assert!(previews.wanted.is_empty());
        assert!(context.tex_manager().read().meta(old_id).is_none());
        assert!(cancelled.load(Ordering::Relaxed));
        sender.send(Some(vec![255; (LIBRARY_PREVIEW_SIZE * LIBRARY_PREVIEW_SIZE * 4) as usize])).unwrap();
        previews.collect(&context, &library, false);
        assert!(previews.active.is_none());
        assert!(previews.textures.is_empty(), "the old worker result must be discarded");
        previews.native(&context, "levels");
        previews.collect(&context, &library, false);
        #[cfg(not(target_arch = "wasm32"))]
        assert_eq!(previews.active.as_ref().unwrap().key, Key::Native("levels".into()));
        assert!(previews.wanted.is_empty(), "clearing must not schedule the entire native catalog");
    }

    #[test]
    fn editing_takes_priority_and_dropping_preview_state_cancels_work() {
        let context = egui::Context::default();
        let library = Arc::new(NodeLibrary::default());
        let mut previews = GraphPreviews::default();
        previews.get(&context, &library, "test", "graph");
        previews.collect(&context, &library, true);
        assert!(previews.active.is_none());
        let (job, _sender) = active(library.clone(), Key::Graph("test".into(), "graph".into()));
        let cancelled = job.cancel.clone();
        previews.active = Some(job);
        drop(previews);
        assert!(cancelled.load(Ordering::Relaxed));
    }
}

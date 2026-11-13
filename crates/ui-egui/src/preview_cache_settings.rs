//! Shared limits for existing workers and one background maintenance job at a time.
use materialcraft_engine::LibraryCachePrefs;
use std::sync::{
    Arc, Mutex,
    mpsc::{Receiver, TryRecvError},
};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Limits {
    pub budget: u64,
    pub entries: usize,
}

impl From<LibraryCachePrefs> for Limits {
    fn from(prefs: LibraryCachePrefs) -> Self {
        let prefs = prefs.sanitized();
        Self { budget: u64::from(prefs.disk_mib) * 1024 * 1024, entries: prefs.max_previews as usize }
    }
}

#[derive(Clone, Copy)]
pub(super) struct State {
    pub limits: Limits,
    pub revision: u64,
    pub epoch: u64,
    pub clear_pending: bool,
    pub disabled: bool,
}

#[derive(Clone)]
pub(super) struct Settings(Arc<Mutex<State>>);

impl Settings {
    pub fn new(prefs: LibraryCachePrefs) -> Self {
        Self(Arc::new(Mutex::new(State { limits: prefs.into(), revision: 0, epoch: 0, clear_pending: false, disabled: false })))
    }

    pub fn get(&self) -> State {
        *self.0.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    pub fn update(&self, limits: Limits, clear: bool) {
        let mut state = self.0.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if state.limits != limits || clear {
            state.limits = limits;
            state.revision = state.revision.wrapping_add(1);
            state.disabled = false;
            if clear {
                state.epoch = state.epoch.wrapping_add(1);
                state.clear_pending = true;
            }
        }
    }

    pub fn finish(&self, epoch: u64, failed: bool) {
        let mut state = self.0.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if state.epoch == epoch {
            state.clear_pending = false;
            // If cleanup fails, rebuild from source instead of showing files we could not clear.
            state.disabled = failed;
        }
    }
}

pub(super) struct Maintenance {
    pub settings: Settings,
    active: Option<Receiver<std::io::Result<Option<State>>>>,
    applied: Option<u64>,
    retry_at: Option<Instant>,
}

impl Maintenance {
    pub fn new(prefs: LibraryCachePrefs) -> Self {
        Self { settings: Settings::new(prefs), active: None, applied: None, retry_at: None }
    }

    pub fn tick(&mut self, ctx: &egui::Context) -> Option<String> {
        let mut error = None;
        if let Some(active) = &self.active {
            let result = match active.try_recv() {
                Ok(result) => Some(result),
                Err(TryRecvError::Empty) => None,
                Err(TryRecvError::Disconnected) => Some(Err(std::io::Error::other("preview cache worker stopped"))),
            };
            if let Some(result) = result {
                self.active = None;
                match result {
                    Ok(Some(state)) => {
                        self.applied = Some(state.revision);
                        self.settings.finish(state.epoch, false);
                    }
                    Ok(None) => self.retry_at = Some(Instant::now() + Duration::from_millis(100)),
                    Err(reason) => {
                        let state = self.settings.get();
                        self.applied = Some(state.revision);
                        self.settings.finish(state.epoch, true);
                        error = Some(format!("Library preview cache unavailable: {reason}"));
                    }
                }
            }
        }
        if self.active.is_none() && self.applied != Some(self.settings.get().revision) {
            if let Some(delay) = self.retry_at.and_then(|when| when.checked_duration_since(Instant::now())) {
                ctx.request_repaint_after(delay);
                return error;
            }
            self.retry_at = None;
            let settings = self.settings.clone();
            let context = ctx.clone();
            let (sender, receiver) = std::sync::mpsc::sync_channel(1);
            match std::thread::Builder::new().name("materialcraft-preview-cache".into()).spawn(move || {
                let state = settings.get();
                let result = match super::preview_disk_cache::PreviewDiskCache::configured(settings) {
                    Some(cache) => cache.maintain(state).map(|done| done.then_some(state)),
                    None => Ok(Some(state)),
                };
                let _ = sender.send(result);
                context.request_repaint();
            }) {
                Ok(_) => self.active = Some(receiver),
                Err(reason) => {
                    let state = self.settings.get();
                    self.applied = Some(state.revision);
                    self.settings.finish(state.epoch, true);
                    error = Some(format!("Library preview cache worker unavailable: {reason}"));
                }
            }
        }
        error
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_old_cleanup_cannot_finish_a_new_clear_request() {
        let settings = Settings::new(LibraryCachePrefs::default());
        settings.update(settings.get().limits, true);
        let old = settings.get();
        settings.update(LibraryCachePrefs { disk_mib: 128, max_previews: 8192 }.into(), true);
        settings.finish(old.epoch, false);
        assert!(settings.get().clear_pending);
        let latest = settings.get();
        settings.finish(latest.epoch, true);
        assert!(!settings.get().clear_pending, "failed cleanup must allow rendering to resume");
        assert!(settings.get().disabled, "failed cleanup must bypass old files");
    }

    #[test]
    fn rapid_changes_keep_one_maintenance_worker_and_coalesce_the_latest_request() {
        let context = egui::Context::default();
        let mut maintenance = Maintenance::new(LibraryCachePrefs::default());
        let (sender, receiver) = std::sync::mpsc::sync_channel(1);
        maintenance.active = Some(receiver);
        for size in 1..=100 {
            maintenance.settings.update(LibraryCachePrefs { disk_mib: size, max_previews: 4096 }.into(), true);
            assert!(maintenance.tick(&context).is_none());
            assert!(maintenance.applied.is_none());
        }
        let latest = maintenance.settings.get();
        // The original receiver is still installed; no replacement jobs may have been spawned.
        sender.send(Ok(Some(latest))).unwrap();
        assert!(maintenance.tick(&context).is_none());
        assert!(maintenance.active.is_none());
        assert_eq!(maintenance.applied, Some(latest.revision));
        assert!(!maintenance.settings.get().clear_pending);
    }
}

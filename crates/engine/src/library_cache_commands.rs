use crate::{
    Session, cmd,
    commands::{CommandSpec, always, bad},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// Disposable library thumbnails; independent of package contents and undo history.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct LibraryCachePrefs {
    /// Zero disables disk persistence.
    pub disk_mib: u32,
    pub max_previews: u32,
}

impl Default for LibraryCachePrefs {
    fn default() -> Self {
        Self { disk_mib: 64, max_previews: 4096 }
    }
}

impl LibraryCachePrefs {
    pub const MAX_DISK_MIB: u32 = 65_536;
    // Bound directory scans and metadata allocations even with externally supplied settings.
    pub const MAX_PREVIEWS: u32 = 65_536;

    pub fn sanitized(self) -> Self {
        Self { disk_mib: self.disk_mib.min(Self::MAX_DISK_MIB), max_previews: self.max_previews.clamp(1, Self::MAX_PREVIEWS) }
    }
}

fn limit(parameters: &Value, key: &str, default: u32, min: u32, max: u32) -> crate::Result<u32> {
    match parameters.get(key) {
        None => Ok(default),
        Some(value) => value
            .as_u64()
            .and_then(|v| u32::try_from(v).ok())
            .filter(|v| (min..=max).contains(v))
            .ok_or_else(|| bad("library.cache.configure", format!("{key} must be an integer from {min} to {max}"))),
    }
}

pub(crate) fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(
            "library.cache.configure",
            "Library preview cache preferences",
            [],
            None,
            "{diskMib?: 0..65536, maxPreviews?: 1..65536}; 0 MiB disables persistence; {} reads preferences; does not modify packages",
            always,
            |session, parameters| {
                let prefs = session.library_cache_prefs;
                let disk_mib = limit(parameters, "diskMib", prefs.disk_mib, 0, LibraryCachePrefs::MAX_DISK_MIB)?;
                let max_previews = limit(parameters, "maxPreviews", prefs.max_previews, 1, LibraryCachePrefs::MAX_PREVIEWS)?;
                session.library_cache_prefs = LibraryCachePrefs { disk_mib, max_previews };
                Ok(json!(session.library_cache_prefs))
            }
        ),
        cmd!(
            "library.cache.clear",
            "Clear library preview cache",
            [],
            None,
            "{} requests background disk cleanup and discards loaded library thumbnails; visible previews rebuild on demand in the app",
            always,
            |session: &mut Session, _| {
                session.library_cache_clear_epoch = session.library_cache_clear_epoch.wrapping_add(1);
                Ok(json!({"requested": true}))
            }
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preferences_and_clear_do_not_edit_packages_and_invalid_updates_are_atomic() {
        let mut session = Session::new(crate::core::Package::new("Cache preferences"));
        let history = session.history();
        let modified = session.modified;
        crate::execute(&mut session, "library.cache.configure", &json!({"diskMib": 512, "maxPreviews": 12000})).unwrap();
        let expected = LibraryCachePrefs { disk_mib: 512, max_previews: 12000 };
        assert_eq!(session.library_cache_prefs, expected);
        for parameters in [
            json!({"diskMib": -1}),
            json!({"diskMib": 65537}),
            json!({"diskMib": "64"}),
            json!({"diskMib": 32, "maxPreviews": 0}),
            json!({"maxPreviews": 65537}),
            json!({"diskMib": 1.5}),
        ] {
            assert!(crate::execute(&mut session, "library.cache.configure", &parameters).is_err());
            assert_eq!(session.library_cache_prefs, expected);
        }
        assert_eq!(crate::execute(&mut session, "library.cache.configure", &json!({})).unwrap(), json!(expected));
        crate::execute(&mut session, "library.cache.clear", &json!({})).unwrap();
        assert_eq!(session.library_cache_clear_epoch, 1);
        assert_eq!(session.history(), history);
        assert_eq!(session.modified, modified);
        crate::execute(&mut session, "library.cache.configure", &json!({"diskMib": 0})).unwrap();
        assert_eq!(session.library_cache_prefs.disk_mib, 0);
    }
}

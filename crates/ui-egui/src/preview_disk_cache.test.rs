use super::*;

const SIDE: u32 = 8;
// A fork can briefly inherit another test's file lock before exec closes it. Keep this
// module's subprocess launches apart from its nonblocking-writer tests.
static TESTS: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn cache(root: &Path) -> PreviewDiskCache {
    PreviewDiskCache { root: root.to_owned(), renderer: [1; 32], budget: 64 * 1024 * 1024, entries: MAX_ENTRIES }
}

fn pixels() -> Vec<u8> {
    (0..SIDE * SIDE * 4).map(|i| i as u8).collect()
}

#[test]
fn previews_survive_process_restarts_without_rendering() {
    let _serial = TESTS.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    const CHILD: &str = "MATERIALCRAFT_TEST_DISK_PREVIEW";
    if let Some(root) = std::env::var_os(CHILD) {
        let cache = cache(Path::new(&root));
        let actual = cache.get_or_render([2; 32], SIDE, &AtomicBool::new(false), || {
            assert_eq!(std::env::var("MATERIALCRAFT_TEST_DISK_PHASE").unwrap(), "cold", "warm launch must not cook");
            Some(pixels())
        });
        assert_eq!(actual, Some(pixels()));
        return;
    }
    let root = tempfile::tempdir().unwrap();
    for phase in ["cold", "warm"] {
        let start = std::time::Instant::now();
        let status = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "thumbs::preview_disk_cache::tests::previews_survive_process_restarts_without_rendering", "--nocapture"])
            .env(CHILD, root.path())
            .env("MATERIALCRAFT_TEST_DISK_PHASE", phase)
            .status()
            .unwrap();
        assert!(status.success());
        eprintln!("{phase} preview subprocess: {:?}", start.elapsed());
    }
}

#[test]
#[ignore = "renders bundled library graphs and measures cold/warm subprocesses"]
fn real_library_previews_survive_relaunch() {
    let _serial = TESTS.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    const CHILD: &str = "MATERIALCRAFT_TEST_REAL_PREVIEWS";
    if let Some(root) = std::env::var_os(CHILD) {
        let library = materialcraft_engine::NodeLibrary::shared();
        let phase = std::env::var("MATERIALCRAFT_TEST_DISK_PHASE").unwrap();
        let cache = PreviewDiskCache::open(PathBuf::from(root), 64 * 1024 * 1024).unwrap();
        let cancel = AtomicBool::new(false);
        for (package, graph) in
            [("sbs://blur_hq.sbs", "blur_hq"), ("sbs://noise_clouds_3_v2.sbs", "clouds_3"), ("sbs://materials/pbr/bricks_001.sbs", "bricks_001")]
        {
            let started = std::time::Instant::now();
            let key = library.preview_fingerprint(package, graph, &cancel).unwrap();
            let image = cache.get_or_render(key, 72, &cancel, || {
                assert_eq!(phase, "cold", "warm launch must load {graph} without cooking");
                materialcraft_engine::NodeLibrary::preview_cancellable(&library, package, graph, 72, &cancel)
                    .map(|image| image.resize(72, 72, true).to_rgba8_display())
            });
            assert_eq!(image.unwrap().len(), 72 * 72 * 4);
            eprintln!("{phase} {graph}: {:?}", started.elapsed());
        }
        return;
    }
    let root = tempfile::tempdir().unwrap();
    for phase in ["cold", "warm"] {
        let status = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "thumbs::preview_disk_cache::tests::real_library_previews_survive_relaunch", "--ignored", "--nocapture"])
            .env(CHILD, root.path())
            .env("MATERIALCRAFT_TEST_DISK_PHASE", phase)
            .status()
            .unwrap();
        assert!(status.success());
    }
}

#[test]
fn keys_separate_sources_sizes_and_renderer_builds() {
    let _serial = TESTS.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let root = tempfile::tempdir().unwrap();
    let mut cache = cache(root.path());
    let original = cache.path([1; 32], SIDE);
    assert_ne!(original, cache.path([2; 32], SIDE));
    assert_ne!(original, cache.path([1; 32], SIDE + 1));
    cache.renderer = [2; 32];
    assert_ne!(original, cache.path([1; 32], SIDE));
}

#[test]
fn cancelled_failed_and_invalid_previews_never_persist() {
    let _serial = TESTS.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let root = tempfile::tempdir().unwrap();
    let cache = cache(root.path());
    let cancel = AtomicBool::new(true);
    assert!(cache.get_or_render([1; 32], SIDE, &cancel, || panic!("cancelled cook ran")).is_none());
    cancel.store(false, Ordering::Relaxed);
    assert!(cache.get_or_render([1; 32], SIDE, &cancel, || None).is_none());
    assert!(
        cache
            .get_or_render([1; 32], SIDE, &cancel, || {
                cancel.store(true, Ordering::Relaxed);
                Some(pixels())
            })
            .is_none()
    );
    cache.write(&cache.path([2; 32], SIDE), SIDE, &[1]).unwrap();
    assert_eq!(fs::read_dir(root.path()).unwrap().count(), 0);
}

#[test]
fn corrupt_oversized_and_wrong_size_files_are_misses_and_repaired() {
    let _serial = TESTS.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let root = tempfile::tempdir().unwrap();
    let cache = cache(root.path());
    let path = cache.path([1; 32], SIDE);
    for bytes in [b"broken PNG".to_vec(), vec![0; MAX_FILE_BYTES as usize + 1]] {
        fs::write(&path, bytes).unwrap();
        assert!(cache.read(&path, SIDE).is_none());
        assert_eq!(cache.get_or_render([1; 32], SIDE, &AtomicBool::new(false), || Some(pixels())), Some(pixels()));
        assert_eq!(cache.read(&path, SIDE), Some(pixels()));
        assert!(cache.read(&path, SIDE + 1).is_none());
    }
    let bytes = fs::read(&path).unwrap();
    fs::write(&path, &bytes[..bytes.len() / 2]).unwrap();
    assert!(cache.read(&path, SIDE).is_none());
}

#[test]
fn cache_budget_evicts_old_files_and_preserves_unrelated_files() {
    let _serial = TESTS.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let root = tempfile::tempdir().unwrap();
    let mut cache = cache(root.path());
    let old = cache.path([1; 32], SIDE);
    cache.write(&old, SIDE, &pixels()).unwrap();
    let file_bytes = fs::metadata(&old).unwrap().len();
    cache.budget = file_bytes * 2;
    File::options().write(true).open(&old).unwrap().set_modified(SystemTime::UNIX_EPOCH).unwrap();
    fs::write(root.path().join("my-image.png"), b"unrelated").unwrap();
    let leftover = root.path().join(craft_atomic::temp_name(old.file_name().unwrap().to_str().unwrap()));
    fs::write(&leftover, b"interrupted write").unwrap();
    fs::write(root.path().join("unrelated.craft_tmp"), b"keep").unwrap();
    cache.write(&cache.path([2; 32], SIDE), SIDE, &pixels()).unwrap();
    assert!(!leftover.exists());
    assert!(root.path().join("unrelated.craft_tmp").exists());
    cache.write(&cache.path([3; 32], SIDE), SIDE, &pixels()).unwrap();
    assert!(!old.exists());
    assert_eq!(fs::read(root.path().join("my-image.png")).unwrap(), b"unrelated");
    let total: u64 =
        fs::read_dir(root.path()).unwrap().flatten().filter(|f| cache_name(&f.file_name().to_string_lossy())).map(|f| f.metadata().unwrap().len()).sum();
    assert!(total <= cache.budget);
    cache.budget = 64 * 1024 * 1024;
    cache.entries = 1;
    cache.write(&cache.path([4; 32], SIDE), SIDE, &pixels()).unwrap();
    let count = fs::read_dir(root.path()).unwrap().flatten().filter(|file| cache_name(&file.file_name().to_string_lossy())).count();
    assert_eq!(count, 1, "the entry limit also applies when there is space left");
    cache.budget = file_bytes - 1;
    let oversized = cache.path([5; 32], SIDE);
    cache.write(&oversized, SIDE, &pixels()).unwrap();
    assert!(!oversized.exists());
}

#[test]
fn unavailable_or_busy_cache_still_returns_rendered_pixels() {
    let _serial = TESTS.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let root = tempfile::tempdir().unwrap();
    let blocked = root.path().join("file");
    fs::write(&blocked, b"not a directory").unwrap();
    assert_eq!(cache(&blocked).get_or_render([1; 32], SIDE, &AtomicBool::new(false), || Some(pixels())), Some(pixels()));
    let lock = File::options().read(true).write(true).create_new(true).open(root.path().join("cache.lock")).unwrap();
    lock.lock().unwrap();
    let cache = cache(root.path());
    assert_eq!(cache.get_or_render([1; 32], SIDE, &AtomicBool::new(false), || Some(pixels())), Some(pixels()));
    assert!(!cache.path([1; 32], SIDE).exists());
}

#[cfg(unix)]
#[test]
fn cache_does_not_read_or_evict_symlink_targets() {
    let _serial = TESTS.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let root = tempfile::tempdir().unwrap();
    let outside = tempfile::NamedTempFile::new().unwrap();
    fs::write(outside.path(), b"keep").unwrap();
    let cache = cache(root.path());
    let path = cache.path([1; 32], SIDE);
    std::os::unix::fs::symlink(outside.path(), &path).unwrap();
    assert!(cache.read(&path, SIDE).is_none());
    let _ = cache.write(&path, SIDE, &pixels());
    assert_eq!(fs::read(outside.path()).unwrap(), b"keep");
}

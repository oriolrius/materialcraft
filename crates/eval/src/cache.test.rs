use super::*;
use materialcraft_image::{Image, Kind};

fn output(side: u32) -> Arc<Vec<NodeOutput>> {
    Arc::new(vec![NodeOutput::Image(Image::filled(side, side, Kind::Gray, [0.5; 4]))])
}

#[test]
fn oversized_results_are_not_retained_or_allowed_to_flush_useful_entries() {
    let mut cache = Cache::new(64);
    cache.put(1, output(4));
    let large = output(8);
    let weak = Arc::downgrade(&large);
    cache.put(2, large);
    assert!(weak.upgrade().is_none());
    assert!(cache.get(1).is_some());
    assert!(cache.get(2).is_none());
    assert!(cache.bytes() <= 64);
}

#[test]
fn disabled_and_empty_caches_do_not_retain_outputs_or_metadata() {
    let mut cache = Cache::new(0);
    cache.put(1, output(4));
    assert!(cache.is_empty());
    for key in 0..10_000 {
        cache.put(key, Arc::new(Vec::new()));
    }
    assert!(cache.is_empty());
}

#[test]
fn repeated_edits_release_evicted_buffers_and_keep_recent_hits() {
    let mut cache = Cache::new(128);
    let first = output(4);
    let first_weak = Arc::downgrade(&first);
    cache.put(0, first);
    cache.put(1, output(4));
    assert!(cache.get(0).is_some());
    cache.put(2, output(4));
    assert!(cache.get(1).is_none());
    assert!(first_weak.upgrade().is_some());
    for key in 3..1_000 {
        cache.put(key, output(4));
        assert!(cache.bytes() <= 128);
        assert!(cache.len() <= 2);
    }
    assert!(first_weak.upgrade().is_none());
    cache.clear();
    assert_eq!(cache.bytes(), 0);
    assert!(cache.is_empty());
}

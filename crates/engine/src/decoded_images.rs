use std::collections::{HashMap, HashSet};

use materialcraft_image::Image;

/// Decoded bitmaps are expendable: resources and active cooks own their source and output data.
pub(crate) struct DecodedImages {
    images: HashMap<(u64, bool), (Image, u64)>,
    bytes: usize,
    limit: usize,
    tick: u64,
}

impl Default for DecodedImages {
    fn default() -> Self {
        Self::new(128 << 20)
    }
}

impl DecodedImages {
    pub fn new(limit: usize) -> Self {
        Self { images: HashMap::new(), bytes: 0, limit, tick: 0 }
    }

    pub fn get(&mut self, key: &(u64, bool)) -> Option<Image> {
        self.tick = self.tick.saturating_add(1);
        self.images.get_mut(key).map(|(image, used)| {
            *used = self.tick;
            image.clone()
        })
    }

    pub fn insert(&mut self, key: (u64, bool), image: Image) {
        if let Some((old, _)) = self.images.remove(&key) {
            self.bytes = self.bytes.saturating_sub(bytes(&old));
        }
        let size = bytes(&image);
        // A large decode can serve its current cook without occupying the persistent cache.
        if size == 0 || size > self.limit {
            return;
        }
        while self.bytes > self.limit - size || self.images.len() >= 128 {
            let Some(key) = self.images.iter().min_by_key(|(_, (_, used))| *used).map(|(key, _)| *key) else { break };
            if let Some((old, _)) = self.images.remove(&key) {
                self.bytes = self.bytes.saturating_sub(bytes(&old));
            }
        }
        self.tick = self.tick.saturating_add(1);
        self.images.insert(key, (image, self.tick));
        self.bytes += size;
    }

    pub fn retain_resources(&mut self, hashes: &HashSet<u64>) {
        self.images.retain(|(hash, _), _| hashes.contains(hash));
        self.bytes = self.images.values().map(|(image, _)| bytes(image)).sum();
    }

    pub fn clear(&mut self) {
        self.images.clear();
        self.bytes = 0;
    }
}

fn bytes(image: &Image) -> usize {
    image.data.len().saturating_mul(size_of::<f32>())
}

#[cfg(test)]
mod tests {
    use super::*;
    use materialcraft_image::Kind;
    use std::sync::Arc;

    #[test]
    fn repeated_decodes_are_bounded_and_evicted_images_are_released() {
        let mut cache = DecodedImages::new(128);
        let image = Image::filled(4, 4, Kind::Gray, [0.5; 4]);
        let weak = Arc::downgrade(&image.data);
        cache.insert((0, false), image);
        for key in 1..1_000 {
            cache.insert((key, key % 2 == 0), Image::filled(4, 4, Kind::Gray, [0.5; 4]));
            assert!(cache.bytes <= 128);
            assert!(cache.images.len() <= 2);
        }
        assert!(weak.upgrade().is_none());
        assert!(cache.get(&(999, false)).is_some());
        cache.retain_resources(&HashSet::new());
        assert_eq!(cache.bytes, 0);
        assert!(cache.images.is_empty());
    }

    #[test]
    fn large_images_bypass_the_cache_and_recent_reads_survive_eviction() {
        let mut cache = DecodedImages::new(128);
        for key in 0..2 {
            cache.insert((key, false), Image::filled(4, 4, Kind::Gray, [0.5; 4]));
        }
        assert!(cache.get(&(0, false)).is_some());
        cache.insert((2, false), Image::filled(4, 4, Kind::Gray, [0.5; 4]));
        assert!(cache.get(&(1, false)).is_none());
        let large = Image::filled(16, 16, Kind::Gray, [0.5; 4]);
        let weak = Arc::downgrade(&large.data);
        cache.insert((3, false), large);
        assert!(weak.upgrade().is_none());
        assert!(cache.get(&(0, false)).is_some());
        cache.clear();
        assert_eq!(cache.bytes, 0);
    }
}

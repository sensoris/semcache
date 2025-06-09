use lru::LruCache;
use std::mem::size_of;
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Mutex};
use tracing::error;

struct EntryMetadata {
    size_bytes: usize,
}

struct CacheEntry<T> {
    response: T,
    metadata: EntryMetadata,
}

pub struct ResponseStore<T> {
    cache: Arc<Mutex<LruCache<u64, CacheEntry<T>>>>,
    total_size_bytes: Arc<AtomicUsize>,
}

const MUTEX_PANIC: &str = "Mutex attempted to get grabbed twice by the same thread, unrecoverable error in response_store";

impl<T: Clone + 'static> ResponseStore<T> {
    // Compile-time constant for base entry size
    const BASE_ENTRY_SIZE: usize = size_of::<CacheEntry<T>>();

    // Creates a new ResponseStore for a generic response type. Does not automatically evict
    // items so those operations need to be performed by the orchestrator.
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(LruCache::unbounded())),
            total_size_bytes: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn get(&self, id: u64) -> Option<T> {
        let mut cache = self.cache.lock().unwrap_or_else(|err| {
            error!(error = ?err, "Mutex poisoned");
            panic!("{}", MUTEX_PANIC)
        });
        let response = &cache.get_mut(&id)?.response;
        Some(response.clone())
    }

    pub fn put(&self, id: u64, response: T) {
        let size_bytes = self.calculate_entry_size(&response);
        let entry = CacheEntry {
            response,
            metadata: EntryMetadata { size_bytes },
        };

        let mut cache = self.cache.lock().unwrap_or_else(|err| {
            error!(error = ?err, "Mutex poisoned");
            panic!("{}", MUTEX_PANIC)
        });

        // If the cache contains the id, we need to get its byte_size to maintain the total_size_bytes
        let old_size = cache
            .peek(&id)
            .map(|entry| entry.metadata.size_bytes)
            .unwrap_or(0);

        cache.put(id, entry);

        let size_delta = size_bytes as i64 - old_size as i64;
        if size_delta > 0 {
            self.total_size_bytes
                .fetch_add(size_delta as usize, std::sync::atomic::Ordering::Relaxed);
        } else if size_delta < 0 {
            self.total_size_bytes
                .fetch_sub((-size_delta) as usize, std::sync::atomic::Ordering::Relaxed);
        }
    }

    pub fn pop(&self) -> Option<u64> {
        let mut cache = self.cache.lock().unwrap_or_else(|err| {
            error!(error = ?err, "Mutex poisoned");
            panic!("{}", MUTEX_PANIC)
        });
        if let Some((id, entry)) = cache.pop_lru() {
            self.total_size_bytes.fetch_sub(
                entry.metadata.size_bytes,
                std::sync::atomic::Ordering::Relaxed,
            );
            Some(id)
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        let cache = self.cache.lock().unwrap_or_else(|err| {
            error!(error = ?err, "Mutex poisoned");
            panic!("{}", MUTEX_PANIC)
        });
        cache.len()
    }

    pub fn memory_usage_bytes(&self) -> usize {
        self.total_size_bytes
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    fn calculate_entry_size(&self, response: &T) -> usize {
        use std::any::Any;

        let response_size =
            if let Some(bytes_response) = (response as &dyn Any).downcast_ref::<Vec<u8>>() {
                bytes_response.len() // actual byte vector content size
            } else {
                error!("Response type not supported");
                size_of::<T>() // fallback to type size for other types
            };

        Self::BASE_ENTRY_SIZE + response_size
    }
}

#[cfg(test)]
mod tests {
    use super::ResponseStore;

    #[test]
    fn put_and_get() {
        let cache = ResponseStore::new();
        let answer = b"The capital of France is Paris.".to_vec();
        cache.put(1, answer.clone());

        let response = cache.get(1).unwrap();
        assert_eq!(answer, response);
    }

    #[test]
    fn pop_removes_lru_entry() {
        let cache = ResponseStore::new();
        cache.put(1, b"first".to_vec());
        cache.put(2, b"second".to_vec());
        cache.put(3, b"third".to_vec());

        cache.get(1);
        cache.get(3);

        let popped = cache.pop().unwrap();
        assert_eq!(popped, 2);
        assert!(cache.get(2).is_none());
    }

    #[test]
    fn pop_returns_none_when_empty() {
        let cache: ResponseStore<Vec<u8>> = ResponseStore::new();
        assert_eq!(cache.pop(), None);
    }

    #[test]
    fn len_tracks_entries() {
        let cache = ResponseStore::new();
        assert_eq!(cache.len(), 0);

        cache.put(1, b"one".to_vec());
        assert_eq!(cache.len(), 1);

        cache.put(2, b"two".to_vec());
        cache.put(3, b"three".to_vec());
        assert_eq!(cache.len(), 3);

        cache.pop();
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn memory_usage_increases_with_entries() {
        let cache = ResponseStore::new();
        let initial_memory = cache.memory_usage_bytes();

        cache.put(1, vec![b'A'; 100]);
        let after_one = cache.memory_usage_bytes();
        assert!(after_one > initial_memory);

        cache.put(2, vec![b'B'; 200]);
        let after_two = cache.memory_usage_bytes();
        assert!(after_two > after_one);
    }

    #[test]
    fn memory_usage_decreases_after_pop() {
        let cache = ResponseStore::new();
        cache.put(1, vec![b'A'; 1000]);
        cache.put(2, vec![b'B'; 1000]);

        let before_pop = cache.memory_usage_bytes();
        cache.pop();
        let after_pop = cache.memory_usage_bytes();

        assert!(after_pop < before_pop);
    }

    #[test]
    fn put_existing_key_updates_memory_usage_correctly() {
        let cache = ResponseStore::new();

        // Put initial entry with small size
        cache.put(1, b"small".to_vec());
        let after_small = cache.memory_usage_bytes();
        assert_eq!(cache.len(), 1);

        // Put same key with larger size - should replace, not add
        cache.put(1, "much_larger_string".repeat(100).into_bytes());
        let after_large = cache.memory_usage_bytes();
        assert_eq!(cache.len(), 1); // Length should stay the same
        assert!(after_large > after_small); // Memory should increase

        // Put same key with smaller size - should decrease memory
        cache.put(1, b"tiny".to_vec());
        let after_tiny = cache.memory_usage_bytes();
        assert_eq!(cache.len(), 1); // Length should stay the same
        assert!(after_tiny < after_large); // Memory should decrease from large
    }
}

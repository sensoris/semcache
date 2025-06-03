use crate::metrics::metrics::CACHE_SIZE;
use lru::LruCache;
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
        let size_bytes = calculate_entry_size(&response);
        let entry = CacheEntry {
            response,
            metadata: EntryMetadata { size_bytes },
        };

        let mut cache = self.cache.lock().unwrap_or_else(|err| {
            error!(error = ?err, "Mutex poisoned");
            panic!("{}", MUTEX_PANIC)
        });
        cache.put(id, entry);
        self.total_size_bytes
            .fetch_add(size_bytes, std::sync::atomic::Ordering::Relaxed);
        CACHE_SIZE.inc()
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
            CACHE_SIZE.dec();
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
}

// TODO (v0): placeholder function while we evaluate better methods
fn calculate_entry_size<T: Clone + 'static>(response: &T) -> usize {
    use std::any::Any;
    use std::mem;

    // Base size of the CacheEntry struct
    let base_size = mem::size_of::<CacheEntry<T>>();

    // For Vec<u8> types, calculate actual byte vector content size
    let response_size =
        if let Some(bytes_response) = (response as &dyn Any).downcast_ref::<Vec<u8>>() {
            bytes_response.len() // actual byte vector content size
        } else {
            mem::size_of::<T>() // fallback to type size for other types
        };

    base_size + response_size
}

#[cfg(test)]
mod tests {
    use super::ResponseStore;

    #[test]
    fn put_and_get() {
        let cache = ResponseStore::new();
        let answer = "The capital of France is Paris.";
        cache.put(1, answer);

        let response = cache.get(1).unwrap();
        assert_eq!(answer, response);
    }

    #[test]
    fn pop_removes_lru_entry() {
        let cache = ResponseStore::new();
        cache.put(1, "first");
        cache.put(2, "second");
        cache.put(3, "third");

        cache.get(1);
        cache.get(3);

        let popped = cache.pop().unwrap();
        assert_eq!(popped, 2);
        assert!(cache.get(2).is_none());
    }

    #[test]
    fn pop_returns_none_when_empty() {
        let cache: ResponseStore<String> = ResponseStore::new();
        assert_eq!(cache.pop(), None);
    }

    #[test]
    fn len_tracks_entries() {
        let cache = ResponseStore::new();
        assert_eq!(cache.len(), 0);

        cache.put(1, "one");
        assert_eq!(cache.len(), 1);

        cache.put(2, "two");
        cache.put(3, "three");
        assert_eq!(cache.len(), 3);

        cache.pop();
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn memory_usage_increases_with_entries() {
        let cache = ResponseStore::new();
        let initial_memory = cache.memory_usage_bytes();

        cache.put(1, "A".repeat(100));
        let after_one = cache.memory_usage_bytes();
        assert!(after_one > initial_memory);

        cache.put(2, "B".repeat(200));
        let after_two = cache.memory_usage_bytes();
        assert!(after_two > after_one);
    }

    #[test]
    fn memory_usage_decreases_after_pop() {
        let cache = ResponseStore::new();
        cache.put(1, "A".repeat(1000));
        cache.put(2, "B".repeat(1000));

        let before_pop = cache.memory_usage_bytes();
        cache.pop();
        let after_pop = cache.memory_usage_bytes();

        assert!(after_pop < before_pop);
    }
}

use std::sync::atomic::{AtomicU64, Ordering};

use super::cache::Cache;
use super::error::CacheError;
use super::semantic_store::semantic_store::SemanticStore;
use crate::cache::response_store::ResponseStore;
use crate::metrics::metrics::CACHE_SIZE;
use tracing::{debug, info};

#[derive(Debug, Clone)]
pub enum EvictionPolicy {
    EntryLimit(usize),
    MemoryLimitMb(usize), // Could also implement a "combined" of both limits
}

const TOP_K: usize = 1;
// set similarity_threshold to 0.99 to allow for floating point rounding
const EXACT_MATCH_SIMILARITY: f32 = 0.99;

pub struct CacheImpl<T> {
    similarity_threshold: f32,
    response_store: ResponseStore<T>,
    semantic_store: Box<dyn SemanticStore>,
    id_generator: AtomicU64,
    eviction_policy: EvictionPolicy,
}

impl<T> CacheImpl<T>
where
    T: Clone + Send + Sync + 'static,
{
    pub fn new(
        semantic_store: Box<dyn SemanticStore>,
        response_store: ResponseStore<T>,
        similarity_threshold: f32,
        eviction_policy: EvictionPolicy,
    ) -> Self {
        assert!(
            (0.0..=1.0).contains(&similarity_threshold),
            "similarity_threshold must be between 0.0 and 1.0"
        );

        let id_generator = AtomicU64::new(0);

        Self {
            similarity_threshold,
            response_store,
            semantic_store,
            id_generator,
            eviction_policy,
        }
    }

    fn is_full(&self) -> bool {
        match &self.eviction_policy {
            EvictionPolicy::EntryLimit(limit) => {
                debug!(
                    "Cache size: {}, limit: {}",
                    self.response_store.len(),
                    limit
                );
                self.response_store.len() >= *limit
            }
            EvictionPolicy::MemoryLimitMb(limit) => {
                let response_store_memory_used_mb =
                    self.response_store.memory_usage_bytes() as f64 / (1024.0 * 1024.0);
                let semantic_store_memory_used_mb =
                    self.semantic_store.memory_usage_bytes() as f64 / (1024.0 * 1024.0);
                let total_memory_used_mb =
                    response_store_memory_used_mb + semantic_store_memory_used_mb;
                let limit_mb = *limit as f64;
                debug!("Cache size: {}, limit: {}", total_memory_used_mb, limit_mb);
                total_memory_used_mb >= limit_mb
            }
        }
    }
}

impl<T> Cache<T> for CacheImpl<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn get_if_present(&self, embedding: &[f32]) -> Result<Option<T>, CacheError> {
        // search semantic store for vectors similar to our query vector
        let search_result = self
            .semantic_store
            .get(embedding, TOP_K, self.similarity_threshold)?;

        // return early if no fitting match found
        if search_result.is_empty() {
            return Ok(None);
        }
        // choose best match
        let id = search_result[0];

        // extract saved response using the index of the nearest vector
        let cached_response = self.response_store.get(id);

        Ok(cached_response)
    }

    fn insert(&self, embedding: Vec<f32>, response: T) -> Result<(), CacheError> {
        let id = self.id_generator.fetch_add(1, Ordering::Relaxed);

        self.response_store.put(id, response);
        self.semantic_store.put(id, embedding)?;

        // Evict entries if policy limits are exceeded
        // todo maybe this should just trigger an idempotent background job to initiate eviction?
        while self.is_full() {
            info!("cache is full, evicting!");
            if let Some(evicted_id) = self.response_store.pop() {
                self.semantic_store.delete(evicted_id)?;
            } else {
                break; // No more entries to evict
            }
        }
        CACHE_SIZE.set(self.response_store.len() as i64);
        debug!("Cache size: {}", self.response_store.len());
        Ok(())
    }

    // checks cache for an exact match, if it finds one it updates the response_store of found id
    // with new body and returns true, otherwise it returns false
    fn try_update(&self, embedding: &[f32], response: T) -> Result<bool, CacheError> {
        let maybe_existing_id: Option<u64> = match self
            .semantic_store
            .get(embedding, 1, EXACT_MATCH_SIMILARITY)?
            .as_slice()
        {
            [] => return Ok(false),
            [head, ..] => Some(*head),
        };
        if let Some(id) = maybe_existing_id {
            self.response_store.put(id, response)
        }
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use faiss::error::Error;
    use mockall::predicate::eq;

    use crate::cache::cache::Cache;
    use crate::cache::cache_impl::{EXACT_MATCH_SIMILARITY, EvictionPolicy};
    use crate::cache::response_store::ResponseStore;
    use crate::cache::{
        cache_impl::{CacheImpl, TOP_K},
        error::CacheError,
        semantic_store::semantic_store::MockSemanticStore,
    };

    // GET

    #[test]
    fn get_should_return_first_entry_when_multiple_found() {
        let embedding = vec![0_f32, 1.0, 0.0];
        let saved_response = String::from("this is a saved response");

        let mut mock_semantic_store = MockSemanticStore::new();
        mock_semantic_store
            .expect_get()
            .with(eq(embedding.clone()), eq(TOP_K), eq(0.9))
            .return_once(|_, _, _| Ok(vec![0, 1, 2]));

        let response_store = ResponseStore::new();
        response_store.put(0, saved_response.clone());

        let under_test = CacheImpl::new(
            Box::new(mock_semantic_store),
            response_store,
            0.9,
            EvictionPolicy::EntryLimit(100),
        );

        // when
        let response = under_test.get_if_present(&embedding).unwrap();

        // then
        assert_eq!(response.unwrap(), saved_response);
    }

    #[test]
    fn get_should_return_empty_when_none_found() {
        let embedding = vec![0_f32, 1.0, 0.0];

        // given
        let mut mock_semantic_store = MockSemanticStore::new();
        mock_semantic_store
            .expect_get()
            .with(eq(embedding.clone()), eq(TOP_K), eq(0.9))
            .return_once(|_, _, _| Ok(vec![]));

        let under_test: CacheImpl<String> = CacheImpl::new(
            Box::new(mock_semantic_store),
            ResponseStore::new(),
            0.9,
            EvictionPolicy::EntryLimit(100),
        );

        // when
        let response = under_test.get_if_present(&embedding).unwrap();

        // then
        assert!(match response {
            Some(_) => panic!("should be empty"),
            None => true,
        });
    }

    #[test]
    fn get_should_return_error_on_semantic_store_failure() {
        let embedding = vec![0.1, 0.2, 0.3];

        // given
        let mut mock_semantic_store = MockSemanticStore::new();
        mock_semantic_store
            .expect_get()
            .with(eq(embedding.clone()), eq(TOP_K), eq(0.9))
            .return_once(|_, _, _| Err(CacheError::FaissRetrievalError(Error::ParameterName)));

        let cache: CacheImpl<String> = CacheImpl::new(
            Box::new(mock_semantic_store),
            ResponseStore::new(),
            0.9,
            EvictionPolicy::EntryLimit(100),
        );

        // when
        let result = cache.get_if_present(&embedding);

        // then
        match result {
            Err(CacheError::FaissRetrievalError(err)) => {
                assert_eq!(err, Error::ParameterName);
            }
            _ => panic!("Expected SemanticStoreError, got {:?}", result),
        }
    }

    // INSERT

    #[test]
    fn insert_should_update_semantic_store_and_insert() {
        let embedding = vec![0.1, 0.2, 0.3];
        let response = String::from("stored response");

        // given

        let mut mock_store = MockSemanticStore::new();
        mock_store
            .expect_put()
            .with(eq(0u64), eq(embedding.clone()))
            .return_once(|_, _| Ok(()));

        let response_store = ResponseStore::new();

        let cache = CacheImpl::new(
            Box::new(mock_store),
            response_store,
            0.9,
            EvictionPolicy::EntryLimit(100),
        );

        // when
        let result = cache.insert(embedding, response.clone());

        // then
        assert!(result.is_ok());

        let stored = cache.response_store.get(0).unwrap();
        assert_eq!(stored.as_str(), response);
    }

    #[test]
    fn insert_should_evict_when_entry_limit_reached() {
        let embedding = vec![0.1_f32, 0.2, 0.3];
        let response = String::from("test response");

        // given
        let mut mock_store = MockSemanticStore::new();
        mock_store.expect_put().times(3).returning(|_, _| Ok(()));
        mock_store.expect_delete().times(2).returning(|_| Ok(()));

        let response_store = ResponseStore::new();

        let cache = CacheImpl::new(
            Box::new(mock_store),
            response_store,
            0.9,
            EvictionPolicy::EntryLimit(2),
        );

        // when - add first entry
        cache.insert(embedding.clone(), response.clone()).unwrap();
        assert_eq!(cache.response_store.len(), 1);
        assert!(!cache.is_full());

        // when - add second entry, this triggers eviction because after adding we have 2 items (which is >= limit)
        cache.insert(embedding.clone(), response.clone()).unwrap();
        assert_eq!(cache.response_store.len(), 1); // evicted back to 1

        // when - add third entry, again triggers eviction
        cache.insert(embedding.clone(), response.clone()).unwrap();
        assert_eq!(cache.response_store.len(), 1); // still 1

        // verify is_full returns false now since we have 1 item and limit is 2
        assert!(!cache.is_full());
    }

    #[test]
    fn insert_should_evict_when_memory_limit_reached() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicUsize, Ordering};

        let embedding = vec![0.1, 0.2, 0.3];
        let response = "A".repeat(400 * 1024).into_bytes(); // 400KB

        // Track number of entries in semantic store for realistic memory reporting
        let entry_count = Arc::new(AtomicUsize::new(0));
        let entry_count_clone = Arc::clone(&entry_count);
        let entry_count_clone2 = Arc::clone(&entry_count);

        // given
        let mut mock_store = MockSemanticStore::new();

        mock_store.expect_put().times(3).returning(move |_, _| {
            entry_count_clone.fetch_add(1, Ordering::Relaxed);
            Ok(())
        });
        mock_store.expect_delete().times(2).returning(move |_| {
            entry_count_clone2.fetch_sub(1, Ordering::Relaxed);
            Ok(())
        });
        mock_store
            .expect_memory_usage_bytes()
            .returning(move || entry_count.load(Ordering::Relaxed) * 400 * 1024); // 400KB per entry

        let response_store = ResponseStore::new();

        // Set limit to 1MB - each entry uses ~0.8MB (400KB response + 400KB semantic)
        let cache = CacheImpl::new(
            Box::new(mock_store),
            response_store,
            0.9,
            EvictionPolicy::MemoryLimitMb(1),
        );

        // when - add first entry
        cache.insert(embedding.clone(), response.clone()).unwrap();
        assert!(!cache.is_full()); // should have ~0.8MB which is under 1MB limit

        // when - add second entry, this should trigger eviction because 2 entries would be ~1.6MB
        cache.insert(embedding.clone(), response.clone()).unwrap();
        assert_eq!(cache.response_store.len(), 1); // evicted back to 1
        assert!(!cache.is_full()); // single entry is under limit

        // when - add third entry, again triggers eviction
        cache.insert(embedding.clone(), response.clone()).unwrap();
        assert_eq!(cache.response_store.len(), 1); // still 1

        // verify cache is not full after eviction
        assert!(!cache.is_full());
    }

    // TRY_UPDATE

    #[test]
    fn try_update_should_update_response_store_with_id_when_present() {
        let embedding = vec![0.1, 0.2, 0.3];
        let response = String::from("new_response");
        let existing_id = 0;

        // given
        let mut mock_store = MockSemanticStore::new();
        mock_store
            .expect_get()
            .with(eq(embedding.clone()), eq(1), eq(EXACT_MATCH_SIMILARITY))
            .return_once(move |_, _, _| Ok(vec![existing_id]));

        let response_store = ResponseStore::new();
        response_store.put(existing_id, String::from("old_response"));

        let cache = CacheImpl::new(
            Box::new(mock_store),
            response_store,
            0.9,
            EvictionPolicy::EntryLimit(100),
        );

        // when
        let result = cache.try_update(&embedding, response.clone()).unwrap();

        // then
        assert!(result);

        let stored = cache.response_store.get(existing_id).unwrap();
        assert_eq!(stored.as_str(), response);
    }

    #[test]
    fn try_update_should_return_false_when_not_present() {
        let embedding = vec![0.1, 0.2, 0.3];
        let new_response = String::from("new_response");
        let existing_id = 0;

        // given
        let mut mock_store = MockSemanticStore::new();
        mock_store
            .expect_get()
            .with(eq(embedding.clone()), eq(1), eq(EXACT_MATCH_SIMILARITY))
            .return_once(move |_, _, _| Ok(vec![]));

        let response_store = ResponseStore::new();

        let cache = CacheImpl::new(
            Box::new(mock_store),
            response_store,
            0.9,
            EvictionPolicy::EntryLimit(100),
        );

        // when
        let result = cache.try_update(&embedding, new_response.clone()).unwrap();

        // then
        assert!(!result);

        let stored = cache.response_store.get(existing_id);
        assert_eq!(stored, None);
    }

    #[test]
    fn cache_size_metric_tracks_correctly() {
        // Setup cache
        let mut mock_store = MockSemanticStore::new();
        mock_store.expect_put().times(2).returning(|_, _| Ok(()));
        mock_store
            .expect_get()
            .times(1)
            .returning(|_, _, _| Ok(vec![0]));

        let cache = CacheImpl::new(
            Box::new(mock_store),
            ResponseStore::new(),
            0.9,
            EvictionPolicy::EntryLimit(100),
        );

        // Insert first entry - should increment len
        cache
            .insert(vec![0.1, 0.2, 0.3], "first response".to_string())
            .unwrap();
        assert_eq!(cache.response_store.len(), 1);

        // Insert second entry - should increment len again
        cache
            .insert(vec![0.4, 0.5, 0.6], "second response".to_string())
            .unwrap();
        assert_eq!(cache.response_store.len(), 2);

        // Try update (overwrite) - should NOT change len
        cache
            .try_update(&[0.1, 0.2, 0.3], "new response".to_string())
            .unwrap();
        assert_eq!(cache.response_store.len(), 2);
    }
}

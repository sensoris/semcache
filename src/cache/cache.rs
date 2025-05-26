use std::sync::atomic::{AtomicU64, Ordering};

use super::error::CacheError;
use super::semantic_store::semantic_store::SemanticStore;
use crate::cache::response_store::ResponseStore;
use tracing::info;

#[derive(Debug, Clone)]
pub enum EvictionPolicy {
    EntryLimit(usize),
    #[allow(dead_code)]
    MemoryLimitBytes(usize), // Could also implement a "combined" of both limits
}

const TOP_K: usize = 1;

pub struct Cache<T> {
    similarity_threshold: f32,
    response_store: ResponseStore<T>,
    semantic_store: Box<dyn SemanticStore>,
    id_generator: AtomicU64,
    eviction_policy: EvictionPolicy,
}

impl<T: Clone + 'static> Cache<T> {
    pub fn new(
        semantic_store: Box<dyn SemanticStore>,
        response_store: ResponseStore<T>,
        similarity_threshold: f32,
        eviction_policy: EvictionPolicy,
    ) -> Self {
        assert!(
            similarity_threshold >= -1.0 && similarity_threshold <= 1.0,
            "similarity_threshold must be between -1.0 and 1.0"
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

    pub fn get_if_present(&self, embedding: &Vec<f32>) -> Result<Option<T>, CacheError> {
        // search semantic store for vectors similar to our query vector
        let search_result =
            self.semantic_store
                .get(&embedding, TOP_K, self.similarity_threshold)?;

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

    pub fn put(&self, embedding: Vec<f32>, response: T) -> Result<(), CacheError> {
        let id = self.id_generator.fetch_add(1, Ordering::Relaxed);

        self.response_store.put(id.into(), response);
        self.semantic_store.put(id.into(), embedding)?;

        // Evict entries if policy limits are exceeded
        // TODO (v0): is a while best way to do this? e.g release chunks of memory before checking
        while self.is_full() {
            info!("CACHE IS FULL, EVICTING!");
            if let Some(evicted_id) = self.response_store.pop() {
                info!("Evicting #{evicted_id}");
                self.semantic_store.delete(evicted_id)?;
            } else {
                break; // No more entries to evict
            }
        }

        Ok(())
    }

    fn is_full(&self) -> bool {
        match &self.eviction_policy {
            EvictionPolicy::EntryLimit(limit) => self.response_store.len() >= *limit,
            EvictionPolicy::MemoryLimitBytes(limit) => {
                let total_memory = self.response_store.memory_usage_bytes()
                    + self.semantic_store.memory_usage_bytes();
                total_memory >= *limit
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use faiss::error::Error;
    use mockall::predicate::eq;

    use crate::cache::cache::EvictionPolicy;
    use crate::cache::response_store::ResponseStore;
    use crate::cache::{
        cache::{Cache, TOP_K},
        error::CacheError,
        semantic_store::semantic_store::MockSemanticStore,
    };

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

        let under_test = Cache::new(
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

        let under_test: Cache<String> = Cache::new(
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
    fn put_should_update_semantic_store_and_insert() {
        let embedding = vec![0.1, 0.2, 0.3];
        let response = String::from("stored response");

        // given

        let mut mock_store = MockSemanticStore::new();
        mock_store
            .expect_put()
            .with(eq(0u64), eq(embedding.clone()))
            .return_once(|_, _| Ok(()));

        let response_store = ResponseStore::new();

        let cache = Cache::new(
            Box::new(mock_store),
            response_store,
            0.9,
            EvictionPolicy::EntryLimit(100),
        );

        // when
        let result = cache.put(embedding, response.clone());

        // then
        assert!(result.is_ok());

        let stored = cache.response_store.get(*&0).unwrap();
        assert_eq!(stored.as_str(), response);
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

        let cache: Cache<String> = Cache::new(
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

    #[test]
    fn put_should_evict_when_entry_limit_reached() {
        let embedding = vec![0.1_f32, 0.2, 0.3];
        let response = String::from("test response");

        // given
        let mut mock_store = MockSemanticStore::new();
        mock_store.expect_put().times(3).returning(|_, _| Ok(()));
        mock_store.expect_delete().times(2).returning(|_| Ok(()));

        let response_store = ResponseStore::new();

        let cache = Cache::new(
            Box::new(mock_store),
            response_store,
            0.9,
            EvictionPolicy::EntryLimit(2),
        );

        // when - add first entry
        cache.put(embedding.clone(), response.clone()).unwrap();
        assert_eq!(cache.response_store.len(), 1);
        assert!(!cache.is_full());

        // when - add second entry, this triggers eviction because after adding we have 2 items (which is >= limit)
        cache.put(embedding.clone(), response.clone()).unwrap();
        assert_eq!(cache.response_store.len(), 1); // evicted back to 1

        // when - add third entry, again triggers eviction
        cache.put(embedding.clone(), response.clone()).unwrap();
        assert_eq!(cache.response_store.len(), 1); // still 1

        // verify is_full returns false now since we have 1 item and limit is 2
        assert!(!cache.is_full());
    }

    #[test]
    fn put_should_evict_when_memory_limit_reached() {
        let embedding = vec![0.1, 0.2, 0.3];
        let response = String::from("A".repeat(100));

        // given
        let mut mock_store = MockSemanticStore::new();

        mock_store.expect_put().times(3).returning(|_, _| Ok(()));
        mock_store.expect_delete().times(2).returning(|_| Ok(()));
        mock_store.expect_memory_usage_bytes().returning(|| 50);

        let response_store = ResponseStore::new();

        let cache = Cache::new(
            Box::new(mock_store),
            response_store,
            0.9,
            EvictionPolicy::MemoryLimitBytes(300),
        );

        // when - add first entry
        cache.put(embedding.clone(), response.clone()).unwrap();
        assert!(!cache.is_full()); // should have ~150 bytes (100 string + overhead + 50 semantic)

        // when - add second entry, this triggers eviction because memory exceeds limit of 300
        cache.put(embedding.clone(), response.clone()).unwrap();
        assert_eq!(cache.response_store.len(), 1); // evicted back to 1
        assert!(!cache.is_full());

        // when - add third entry, again triggers eviction
        cache.put(embedding.clone(), response.clone()).unwrap();
        assert_eq!(cache.response_store.len(), 1); // still 1

        // verify cache is not full after eviction
        assert!(!cache.is_full());
    }
}

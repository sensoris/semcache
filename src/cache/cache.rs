use std::sync::atomic::{AtomicU64, Ordering};

use super::error::CacheError;
use super::semantic_store::semantic_store::SemanticStore;
use crate::cache::response_store::ResponseStore;
use crate::embedding::service::EmbeddingService;
use faiss::index::SearchResult;

#[derive(Debug, Clone)]
pub enum EvictionPolicy {
    EntryLimit(usize),
    #[allow(dead_code)]
    MemoryLimitBytes(usize), // Could also implement a "combined" of both limits
}

const TOP_K: usize = 1;

pub struct Cache<T> {
    embedding_service: Box<dyn EmbeddingService>,
    similarity_threshold: f32,
    response_store: ResponseStore<T>,
    semantic_store: Box<dyn SemanticStore>,
    id_generator: AtomicU64,
    eviction_policy: EvictionPolicy,
}

impl<T: Clone + 'static> Cache<T> {
    pub fn new(
        embedding_service: Box<dyn EmbeddingService>,
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
            embedding_service,
            similarity_threshold,
            response_store,
            semantic_store,
            id_generator,
            eviction_policy,
        }
    }

    pub fn get_if_present(&self, prompt: &str) -> Result<Option<T>, CacheError> {
        // generate query vector
        let embedding = self.embedding_service.embed(prompt)?;

        // search semantic store for vectors similar to our query vector
        let search_result = self.semantic_store.get(embedding, TOP_K)?;

        // find idx of highest similarity stored value that is above similarity_threshold
        let id = self.find_nearest_id(&search_result);

        // extract saved response using the index of the nearest vector
        let cached_response = id.and_then(|idx| self.response_store.get(idx));

        Ok(cached_response)
    }

    pub fn put(&self, prompt: &String, response: T) -> Result<(), CacheError> {
        //Todo embedding should be outside of cache
        let vec = self.embedding_service.embed(&prompt)?;
        let id = self.id_generator.fetch_add(1, Ordering::Relaxed);

        self.response_store.put(id.into(), response);
        self.semantic_store.put(id.into(), vec)?;

        // Evict entries if policy limits are exceeded
        // Todo is a while best way to do this? e.g release chunks of memory before checking
        while self.is_full() {
            println!("CACHE IS FULL, EVICTING!");
            if let Some(evicted_id) = self.response_store.pop() {
                println!("Evicting #{evicted_id}");
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

    // todo: maybe this can be in the semantic store
    fn find_nearest_id(&self, search_result: &SearchResult) -> Option<u64> {
        let maybe_idx = search_result
            .distances
            .iter()
            .zip(&search_result.labels)
            .filter_map(|(distance, raw_idx)| raw_idx.get().map(|idx| (*distance, idx)))
            .filter(|(distance, _)| *distance > self.similarity_threshold)
            .max_by(|(d1, _), (d2, _)| d1.partial_cmp(d2).unwrap())
            .map(|(_distance, idx)| idx);
        maybe_idx
    }
}

#[cfg(test)]
mod tests {
    use faiss::{Idx, error::Error, index::SearchResult};
    use mockall::predicate::eq;

    use crate::cache::cache::EvictionPolicy;
    use crate::cache::response_store::ResponseStore;
    use crate::{
        cache::{
            cache::{Cache, TOP_K},
            error::CacheError,
            semantic_store::semantic_store::MockSemanticStore,
        },
        embedding::service::MockEmbeddingService,
    };

    #[test]
    fn get_should_return_most_similar_when_multiple_search_result() {
        let prompt = "give me the cache";
        let embedding = vec![0_f32, 1.0, 0.0];
        let saved_response = String::from("this is a saved response");

        // given
        let mut mock_embedding_service = MockEmbeddingService::new();
        mock_embedding_service
            .expect_embed()
            .with(eq(prompt))
            .return_once({
                let embedding_clone = embedding.clone();
                move |_| Ok(embedding_clone)
            });

        let mut mock_semantic_store = MockSemanticStore::new();
        mock_semantic_store
            .expect_get()
            .with(eq(embedding), eq(TOP_K))
            .return_once(|_, _| {
                Ok(SearchResult {
                    distances: vec![0.8, 0.9, 0.91],
                    labels: vec![Idx::new(0), Idx::new(1), Idx::new(2)],
                })
            });

        let response_store = ResponseStore::new();
        response_store.put(2, saved_response.clone());

        let under_test = Cache::new(
            Box::new(mock_embedding_service),
            Box::new(mock_semantic_store),
            response_store,
            0.9,
            EvictionPolicy::EntryLimit(100),
        );

        // when
        let response = under_test.get_if_present(prompt).unwrap();

        // then
        assert_eq!(response.unwrap(), saved_response);
    }

    #[test]
    fn get_should_return_empty_when_none_found() {
        let prompt = "give me the cache";
        let embedding = vec![0_f32, 1.0, 0.0];

        // given
        let mut mock_embedding_service = MockEmbeddingService::new();
        mock_embedding_service
            .expect_embed()
            .with(eq(prompt))
            .return_once({
                let embedding_clone = embedding.clone();
                move |_| Ok(embedding_clone)
            });

        let mut mock_semantic_store = MockSemanticStore::new();
        mock_semantic_store
            .expect_get()
            .with(eq(embedding), eq(TOP_K))
            .return_once(|_, _| {
                Ok(SearchResult {
                    distances: vec![],
                    labels: vec![],
                })
            });

        let under_test: Cache<String> = Cache::new(
            Box::new(mock_embedding_service),
            Box::new(mock_semantic_store),
            ResponseStore::new(),
            0.9,
            EvictionPolicy::EntryLimit(100),
        );

        // when
        let response = under_test.get_if_present(prompt).unwrap();

        // then
        assert!(match response {
            Some(_) => panic!("should be empty"),
            None => true,
        });
    }

    #[test]
    fn put_should_update_semantic_store_and_insert() {
        let prompt = String::from("index me");
        let embedding = vec![0.1, 0.2, 0.3];
        let response = String::from("stored response");

        // given
        let mut mock_embed = MockEmbeddingService::new();
        mock_embed
            .expect_embed()
            .with(eq(prompt.clone()))
            .return_once({
                let embedding = embedding.clone();
                move |_| Ok(embedding)
            });

        let mut mock_store = MockSemanticStore::new();
        mock_store
            .expect_put()
            .with(eq(0u64), eq(embedding.clone()))
            .return_once(|_, _| Ok(()));

        let response_store = ResponseStore::new();

        let cache = Cache::new(
            Box::new(mock_embed),
            Box::new(mock_store),
            response_store,
            0.9,
            EvictionPolicy::EntryLimit(100),
        );

        // when
        let result = cache.put(&prompt, response.clone());

        // then
        assert!(result.is_ok());

        let stored = cache.response_store.get(*&0).unwrap();
        assert_eq!(stored.as_str(), response);
    }

    #[test]
    fn get_should_filter_out_low_similarity_results() {
        let prompt = "low similarity test";
        let embedding = vec![0.1, 0.2, 0.3];

        // given

        let mut mock_embedding_service = MockEmbeddingService::new();
        mock_embedding_service
            .expect_embed()
            .with(eq(prompt))
            .return_once({
                let embedding = embedding.clone();
                move |_| Ok(embedding)
            });

        let mut mock_semantic_store = MockSemanticStore::new();
        mock_semantic_store
            .expect_get()
            .with(eq(embedding), eq(TOP_K))
            .return_once(|_, _| {
                Ok(SearchResult {
                    distances: vec![0.4, 0.5],
                    labels: vec![Idx::new(0), Idx::new(1)],
                })
            });

        let under_test: Cache<String> = Cache::new(
            Box::new(mock_embedding_service),
            Box::new(mock_semantic_store),
            ResponseStore::new(),
            0.9,
            EvictionPolicy::EntryLimit(100),
        );

        // when
        let result = under_test.get_if_present(prompt).unwrap();

        // then
        assert!(
            result.is_none(),
            "Expected no match due to low similarity scores"
        );
    }

    #[test]
    fn get_should_return_error_on_semantic_store_failure() {
        let prompt = "error test";
        let embedding = vec![0.1, 0.2, 0.3];

        // given
        let mut mock_embedding_service = MockEmbeddingService::new();
        mock_embedding_service
            .expect_embed()
            .with(eq(prompt))
            .return_once({
                let embedding = embedding.clone();
                move |_| Ok(embedding)
            });

        let mut mock_semantic_store = MockSemanticStore::new();
        mock_semantic_store
            .expect_get()
            .with(eq(embedding), eq(TOP_K))
            .return_once(|_, _| Err(CacheError::FaissRetrievalError(Error::ParameterName)));

        let cache: Cache<String> = Cache::new(
            Box::new(mock_embedding_service),
            Box::new(mock_semantic_store),
            ResponseStore::new(),
            0.9,
            EvictionPolicy::EntryLimit(100),
        );

        // when
        let result = cache.get_if_present(prompt);

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
        let prompt = String::from("test prompt");
        let embedding = vec![0.1, 0.2, 0.3];
        let response = String::from("test response");

        // given
        let mut mock_embed = MockEmbeddingService::new();
        mock_embed.expect_embed().times(3).returning({
            let embedding = embedding.clone();
            move |_| Ok(embedding.clone())
        });

        let mut mock_store = MockSemanticStore::new();
        mock_store.expect_put().times(3).returning(|_, _| Ok(()));
        mock_store.expect_delete().times(2).returning(|_| Ok(()));

        let response_store = ResponseStore::new();

        let cache = Cache::new(
            Box::new(mock_embed),
            Box::new(mock_store),
            response_store,
            0.9,
            EvictionPolicy::EntryLimit(2),
        );

        // when - add first entry
        cache.put(&prompt, response.clone()).unwrap();
        assert_eq!(cache.response_store.len(), 1);
        assert!(!cache.is_full());

        // when - add second entry, this triggers eviction because after adding we have 2 items (which is >= limit)
        cache.put(&prompt, response.clone()).unwrap();
        assert_eq!(cache.response_store.len(), 1); // evicted back to 1

        // when - add third entry, again triggers eviction
        cache.put(&prompt, response.clone()).unwrap();
        assert_eq!(cache.response_store.len(), 1); // still 1

        // verify is_full returns false now since we have 1 item and limit is 2
        assert!(!cache.is_full());
    }

    #[test]
    fn put_should_evict_when_memory_limit_reached() {
        let prompt = String::from("test prompt");
        let embedding = vec![0.1, 0.2, 0.3];
        let response = String::from("A".repeat(100));

        // given
        let mut mock_embed = MockEmbeddingService::new();
        mock_embed.expect_embed().times(3).returning({
            let embedding = embedding.clone();
            move |_| Ok(embedding.clone())
        });

        let mut mock_store = MockSemanticStore::new();

        mock_store.expect_put().times(3).returning(|_, _| Ok(()));
        mock_store.expect_delete().times(2).returning(|_| Ok(()));
        mock_store.expect_memory_usage_bytes().returning(|| 50);

        let response_store = ResponseStore::new();

        let cache = Cache::new(
            Box::new(mock_embed),
            Box::new(mock_store),
            response_store,
            0.9,
            EvictionPolicy::MemoryLimitBytes(300),
        );

        // when - add first entry
        cache.put(&prompt, response.clone()).unwrap();
        assert!(!cache.is_full()); // should have ~150 bytes (100 string + overhead + 50 semantic)

        // when - add second entry, this triggers eviction because memory exceeds limit of 300
        cache.put(&prompt, response.clone()).unwrap();
        assert_eq!(cache.response_store.len(), 1); // evicted back to 1
        assert!(!cache.is_full());

        // when - add third entry, again triggers eviction
        cache.put(&prompt, response.clone()).unwrap();
        assert_eq!(cache.response_store.len(), 1); // still 1

        // verify cache is not full after eviction
        assert!(!cache.is_full());
    }
}

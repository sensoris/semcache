use std::sync::atomic::{AtomicU32, Ordering};

use dashmap::DashMap;

use crate::embedding::service::EmbeddingService;

use super::error::CacheError;
use super::semantic_store::semantic_store::SemanticStore;

const TOP_K: usize = 1;
pub struct Cache {
    embedding_service: Box<dyn EmbeddingService>,
    similarity_threshold: f32,
    id_to_response: DashMap<u64, String>,
    semantic_store: Box<dyn SemanticStore>,
    id_generator: AtomicU32,
}

impl Cache {
    pub fn new(
        embedding_service: Box<dyn EmbeddingService>,
        semantic_store: Box<dyn SemanticStore>,
        id_to_response: DashMap<u64, String>,
        similarity_threshold: f32,
    ) -> Self {
        assert!(
            similarity_threshold >= -1.0 && similarity_threshold <= 1.0,
            "similarity_threshold must be between -1.0 and 1.0"
        );
        let id_generator = AtomicU32::new(0);
        Self {
            embedding_service,
            similarity_threshold,
            id_to_response,
            semantic_store,
            id_generator,
        }
    }

    pub fn get_if_present(&self, prompt: &str) -> Result<Option<String>, CacheError> {
        // generate query vector
        let embedding = self.embedding_service.embed(prompt)?;

        // search semantic store for vectors similar to our query vector
        let search_result = self.semantic_store.get(embedding, TOP_K)?;

        // find idx of highest similarity stored value that is above similarity_threshold
        let maybe_idx = search_result
            .distances
            .iter()
            .zip(&search_result.labels)
            .filter_map(|(distance, raw_idx)| raw_idx.get().map(|idx| (*distance, idx)))
            .filter(|(distance, _)| *distance > self.similarity_threshold)
            .max_by(|(d1, _), (d2, _)| d1.partial_cmp(d2).unwrap())
            .map(|(_distance, idx)| idx);

        // extract saved response using the index of the nearest vector
        let saved_response = maybe_idx.and_then(|idx| {
            self.id_to_response
                .get(&idx.into())
                .map(|response| response.clone())
        });

        Ok(saved_response)
    }

    pub fn put(&self, prompt: &String, response: String) -> Result<(), CacheError> {
        let vec = self.embedding_service.embed(&prompt)?;
        let id = self.id_generator.fetch_add(1, Ordering::Relaxed);
        self.semantic_store.put(id.into(), vec)?;
        self.id_to_response.insert(id.into(), response);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use dashmap::DashMap;
    use faiss::{Idx, error::Error, index::SearchResult};
    use mockall::predicate::eq;

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

        let id_to_response = DashMap::new();
        id_to_response.insert(2, saved_response.clone());

        let under_test = Cache::new(
            Box::new(mock_embedding_service),
            Box::new(mock_semantic_store),
            id_to_response,
            0.9,
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

        let id_to_response = DashMap::new();

        let under_test = Cache::new(
            Box::new(mock_embedding_service),
            Box::new(mock_semantic_store),
            id_to_response,
            0.9,
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
            .with(eq(0u32), eq(embedding.clone()))
            .return_once(|_, _| Ok(()));

        let id_to_response: DashMap<u64, String> = DashMap::new();

        let cache = Cache::new(
            Box::new(mock_embed),
            Box::new(mock_store),
            id_to_response,
            0.9,
        );

        // when
        let result = cache.put(&prompt, response.clone());

        // then
        assert!(result.is_ok());

        let stored = cache.id_to_response.get(&0).unwrap();
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

        let id_to_response = DashMap::new();

        let under_test = Cache::new(
            Box::new(mock_embedding_service),
            Box::new(mock_semantic_store),
            id_to_response,
            0.9,
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

        let id_to_response = DashMap::new();

        let cache = Cache::new(
            Box::new(mock_embedding_service),
            Box::new(mock_semantic_store),
            id_to_response,
            0.9,
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
}

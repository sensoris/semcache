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
    use faiss::{Idx, index::SearchResult};
    use mockall::predicate::eq;

    use crate::{
        cache::{cache::{Cache, TOP_K}, semantic_store::semantic_store::MockSemanticStore},
        embedding::service::MockEmbeddingService,
    };

    #[test]
    fn get_should_return_most_similar() {
        // given
        let prompt = "give me the cache";
        let embedding = vec![0_f32, 1.0, 0.0];

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

        let saved_response = String::from("this is a saved response");
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
}

use std::sync::atomic::{AtomicU32, Ordering};

use dashmap::DashMap;

use crate::embedding::{fastembed::FastEmbedService, service::EmbeddingService};

use super::{error::CacheError, semantic_store::SemanticStore};

pub struct Cache {
    embedding_service: FastEmbedService,
    dimensionality: u32,
    similarity_threshold: f32,
    id_to_response: DashMap<u64, String>,
    semantic_store: SemanticStore,
    id_generator: AtomicU32,
}

impl Cache {
    pub fn new(embedding_service: FastEmbedService, similarity_threshold: f32) -> Self {
        assert!(
            similarity_threshold >= -1.0 && similarity_threshold <= 1.0,
            "similarity_threshold must be between -1.0 and 1.0"
        );
        let dimensionality = embedding_service.get_dimensionality();
        let id_to_response = DashMap::new();
        let semantic_store = SemanticStore::new(dimensionality);
        let id_generator = AtomicU32::new(0);
        Self {
            embedding_service,
            dimensionality,
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
        let search_result = self.semantic_store.get(embedding, 1)?;

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

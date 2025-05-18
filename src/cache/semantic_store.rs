use std::sync::RwLock;

use faiss::{
    ConcurrentIndex, IdMap, Index,
    index::{IndexImpl, SearchResult, flat::FlatIndexImpl},
    index_factory,
};

use super::error::CacheError;

pub struct SemanticStore {
    dimensionality: u32,
    faiss_store: RwLock<IdMap<FlatIndexImpl>>,
}

impl SemanticStore {
    pub fn new(dimensionality: u32) -> Self {
        let faiss_index = FlatIndexImpl::new_ip(dimensionality).unwrap();
        let id_map = IdMap::new(faiss_index).unwrap();
        let faiss_store = RwLock::new(id_map);
        SemanticStore {
            dimensionality,
            faiss_store,
        }
    }

    pub fn get(&self, vec: Vec<f32>, top_k: usize) -> Result<SearchResult, CacheError> {
        let read_guard = self
            .faiss_store
            .read()
            .expect("RwLock poisoned, faiss store might be corrputed, panicking!");
        let result = ConcurrentIndex::search(&*read_guard, &vec, top_k)?;
        Ok(result)
    }

    pub fn put(index: u32, val: Vec<f64>) {}
}

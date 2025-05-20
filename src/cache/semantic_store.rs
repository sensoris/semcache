use std::sync::RwLock;

use faiss::{
    ConcurrentIndex, IdMap, Idx, Index,
    index::{SearchResult, flat::FlatIndexImpl},
};

use crate::utils::linear_algebra::normalize;

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
        let vec = normalize(vec);
        let read_guard = self
            .faiss_store
            .read()
            .expect("RwLock poisoned, faiss store might be corrputed, panicking!");
        let result = ConcurrentIndex::search(&*read_guard, &vec, top_k)?;
        Ok(result)
    }

    pub fn put(&self, id: u32, vec: Vec<f32>) -> Result<(), CacheError> {
        let vec = normalize(vec);
        let id = Idx::new(id.into());
        let mut write_guard = self
            .faiss_store
            .write()
            .expect("RwLock poisoned, faiss store might be corrupted, panicking");
        write_guard.add_with_ids(&vec, &vec![id])?;
        Ok(())
    }

    pub fn delete(idx: u32) {
        todo!("semantic_store::delete not implemented")
    }
}

#[cfg(test)]
mod tests {
    use super::SemanticStore;

    #[test]
    fn get_should_return_most_similar() {
        // given
        let cache = SemanticStore::new(3);
        let vec1 = vec![0_f32, 1.0, 0.0];
        let vec2 = vec![0_f32, 0.0, 1.0];
        cache.put(1, vec1).expect("failed to insert vectors");
        cache.put(2, vec2).expect("failed to insert vectors");

        // when
        let query = vec![0_f32, 0.99, 0.0];
        let found = cache.get(query, 1).expect("No vector found");

        // then
        assert_eq!(found.distances.len(), 1);
        assert_eq!(found.labels[0].to_native(), 1);
    }

    #[test]
    fn get_with_identical_vector_should_return_same_vector() {
        // given
        let cache = SemanticStore::new(3);
        let vec1 = vec![0_f32, 1.0, 0.0];
        cache.put(1, vec1).expect("failed to insert vectors");

        // when
        let query = vec![0_f32, 1.0, 0.0];
        let found = cache.get(query, 1).expect("No vector found");

        // then
        assert_eq!(found.distances.len(), 1);
        assert_eq!(found.labels[0].to_native(), 1);
        assert_eq!(found.distances[0], 1.0);
    }
}

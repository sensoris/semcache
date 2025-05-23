use std::sync::RwLock;

use crate::utils::linear_algebra::normalize;
use faiss::selector::IdSelector;
use faiss::{
    ConcurrentIndex, IdMap, Idx, Index,
    index::{SearchResult, flat::FlatIndexImpl},
};

use crate::cache::error::CacheError;

use super::semantic_store::SemanticStore;

pub struct FlatIPFaissStore {
    faiss_store: RwLock<IdMap<FlatIndexImpl>>,
    dimensionality: u32,
}

const RW_LOCK_ERROR: &'static str = "RwLock poisoned, faiss store might be corrupted, panicking";

impl FlatIPFaissStore {
    pub fn new(dimensionality: u32) -> Self {
        let faiss_index = FlatIndexImpl::new_ip(dimensionality).unwrap();
        let id_map = IdMap::new(faiss_index).unwrap();
        let faiss_store = RwLock::new(id_map);
        FlatIPFaissStore {
            faiss_store,
            dimensionality,
        }
    }
}

impl SemanticStore for FlatIPFaissStore {
    fn get(&self, vec: Vec<f32>, top_k: usize) -> Result<SearchResult, CacheError> {
        let vec = normalize(vec);
        let read_guard = self.faiss_store.read().expect(RW_LOCK_ERROR);

        // If index is empty, return empty results
        // todo, does this properly make sense?
        if read_guard.ntotal() == 0 {
            return Ok(SearchResult {
                labels: vec![],
                distances: vec![],
            });
        }

        let result = ConcurrentIndex::search(&*read_guard, &vec, top_k)?;
        Ok(result)
    }

    fn put(&self, id: u64, vec: Vec<f32>) -> Result<(), CacheError> {
        let vec = normalize(vec);
        let id = Idx::new(id.into());
        let mut write_guard = self.faiss_store.write().expect(RW_LOCK_ERROR);
        write_guard.add_with_ids(&vec, &vec![id])?;
        Ok(())
    }

    fn delete(&self, id: u64) -> Result<(), CacheError> {
        let id = Idx::new(id.into());
        let mut write_guard = self
            .faiss_store
            .write()
            .expect("RwLock poisoned, faiss store might be corrupted, panicking");
        let id_sel = IdSelector::batch(&[id])?;
        write_guard.remove_ids(&id_sel)?;
        Ok(())
    }

    // TODO this method is CLAUDE, defo need to evaluate it properly, read online how best to do this
    fn memory_usage_bytes(&self) -> usize {
        let read_guard = self.faiss_store.read().expect(RW_LOCK_ERROR);

        // Each vector takes dimensionality * 4 bytes (f32)
        let vector_size = self.dimensionality as usize * 4;
        let num_vectors = read_guard.ntotal() as usize;
        let raw_vectors_size = num_vectors * vector_size;

        // Add 20% overhead for FAISS index metadata and structures
        (raw_vectors_size as f64 * 1.2) as usize
    }
}

// TODO: fix tests to work with FAISS, e.g with mocks? OR replace with someother vector db...
#[cfg(test)]
mod tests {

    use crate::cache::semantic_store::flat_ip_faiss_store::FlatIPFaissStore;

    use super::SemanticStore;

    #[test]
    fn get_should_return_most_similar() {
        // given
        let faiss_store = FlatIPFaissStore::new(3);
        let vec1 = vec![0_f32, 1.0, 0.0];
        let vec2 = vec![0_f32, 0.0, 1.0];
        faiss_store.put(1, vec1).expect("failed to insert vectors");
        faiss_store.put(2, vec2).expect("failed to insert vectors");

        // when
        let query = vec![0_f32, 0.99, 0.0];
        let found = faiss_store.get(query, 1).expect("No vector found");

        // then
        assert_eq!(found.distances.len(), 1);
        assert_eq!(found.labels[0].to_native(), 1);
    }

    #[test]
    fn get_with_identical_vector_should_return_same_vector() {
        // given
        let cache = FlatIPFaissStore::new(3);
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

    #[test]
    fn delete_should_remove_vector_from_db() {
        let cache = FlatIPFaissStore::new(3);
        let vec1 = vec![0_f32, 1.0, 0.0];
        let id = 1;
        cache.put(id, vec1).expect("");

        let query = vec![0_f32, 0.99, 0.0];

        let found = cache.get(query.clone(), 1).expect("");
        assert_eq!(found.distances.len(), 1);

        cache.delete(id).expect("");
        let after_delete = cache.get(query, 1).expect("");
        assert_eq!(after_delete.distances.len(), 0);
    }
}

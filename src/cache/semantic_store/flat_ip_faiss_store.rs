use std::sync::RwLock;

use crate::utils::linear_algebra::normalize;
use faiss::selector::IdSelector;
use faiss::{
    ConcurrentIndex, IdMap, Idx, Index,
    index::{SearchResult, flat::FlatIndexImpl},
};
use ordered_float::OrderedFloat;
use tracing::error;

use crate::cache::error::CacheError;

use super::semantic_store::SemanticStore;

pub struct FlatIPFaissStore {
    faiss_store: RwLock<IdMap<FlatIndexImpl>>,
    dimensionality: u32,
}

const RW_LOCK_ERROR: &'static str = "RwLock poisoned, faiss store might be corrupted, panicking";

impl FlatIPFaissStore {
    pub fn new(dimensionality: u32) -> Self {
        let faiss_index = FlatIndexImpl::new_ip(dimensionality).unwrap_or_else(|err| {
            error!(error = ?err);
            panic!("failed to init faiss index")
        });
        let id_map = IdMap::new(faiss_index).unwrap_or_else(|err| {
            error!(error = ?err);
            panic!("failed to init faiss index")
        });
        let faiss_store = RwLock::new(id_map);
        FlatIPFaissStore {
            faiss_store,
            dimensionality,
        }
    }
}

impl SemanticStore for FlatIPFaissStore {
    fn get(
        &self,
        vec: &Vec<f32>,
        top_k: usize,
        similarity_threshold: f32,
    ) -> Result<Vec<u64>, CacheError> {
        let similarity_threshold = into_cosine_similarity(similarity_threshold);
        let vec = normalize(&vec);

        let read_guard = self.faiss_store.read().expect(RW_LOCK_ERROR);

        // faiss will return nonsense from a search if it's empty
        if read_guard.ntotal() == 0 {
            return Ok(vec![]);
        }

        let search_result = ConcurrentIndex::search(&*read_guard, &vec, top_k)?;
        let result = find_nearest_ids(search_result, similarity_threshold);
        Ok(result)
    }

    fn put(&self, id: u64, vec: Vec<f32>) -> Result<(), CacheError> {
        let vec = normalize(&vec);
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

    // TODO (v0): this method is CLAUDE, defo need to evaluate it properly, read online how best to do this
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

// takes an internal representation of similarity which is of [0, 1], and rescales it linearly to cosine similarity
fn into_cosine_similarity(similarity_threshold: f32) -> f32 {
    similarity_threshold * 2.0 - 1.0
}

fn find_nearest_ids(search_result: SearchResult, similarity_threshold: f32) -> Vec<u64> {
    let mut distances_and_ids: Vec<(f32, u64)> = search_result
        .distances
        .into_iter()
        // zip the distances and labels together so we can process each vector cohesively
        .zip(search_result.labels.into_iter())
        // filter out NaN distances
        .filter(|(distance, _)| distance.is_finite())
        // filter out invalid id's
        .filter_map(|(distance, idx)| match idx.get() {
            Some(id) => Some((distance, id)),
            None => None,
        })
        // filter out matches that don't meet threshold
        .filter(|(distance, _)| distance >= &similarity_threshold)
        .collect();

    // ensure our found vectors are sorted in order of closest match first
    distances_and_ids.sort_by_key(|(distance, _)| std::cmp::Reverse(OrderedFloat(*distance)));

    // extract ids of matching entries
    distances_and_ids
        .into_iter()
        .map(|distance_and_id| distance_and_id.1)
        .collect()
}

#[cfg(test)]
mod tests {

    use crate::cache::semantic_store::flat_ip_faiss_store::FlatIPFaissStore;

    use super::SemanticStore;

    #[test]
    fn get_should_return_empty_when_store_is_empty() {
        // given
        let faiss_store = FlatIPFaissStore::new(3);

        // when
        let query = vec![0_f32, 0.99, 0.0];
        let found = faiss_store
            .get(&query, 1, 0.9)
            .expect("error in faiss store");

        // then
        assert_eq!(found.len(), 0);
    }

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
        let found = faiss_store.get(&query, 1, 0.9).expect("No vector found");

        // then
        assert_eq!(found.len(), 1);
        assert_eq!(found[0], 1);
    }

    #[test]
    fn get_should_return_closest_vector_first() {
        // given
        let cache = FlatIPFaissStore::new(3);
        let vec1 = vec![0_f32, 0.99, 0.0];
        cache.put(1, vec1).expect("failed to insert vectors");
        let vec2 = vec![0_f32, 1.0, 0.0];
        cache.put(2, vec2).expect("failed to insert vectors");

        // when
        let query = vec![0_f32, 1.0, 0.0];
        let found = cache.get(&query, 2, 0.9).expect("No vector found");

        // then
        assert_eq!(found.len(), 2);
        assert_eq!(found, vec!(2, 1));
    }

    #[test]
    fn get_should_filter_out_matches_below_threshold() {
        // given
        let cache = FlatIPFaissStore::new(3);
        let vec = vec![0_f32, 1.0, 0.0];
        cache.put(1, vec).expect("failed to insert vectors");

        // when
        let query = vec![0_f32, 0.0, 0.0];
        let found = cache.get(&query, 2, 0.9).expect("No vector found");

        // then
        assert_eq!(found.len(), 0);
    }

    #[test]
    fn delete_should_remove_vector_from_db() {
        let cache = FlatIPFaissStore::new(3);
        let vec1 = vec![0_f32, 1.0, 0.0];
        let id = 1;
        cache.put(id, vec1).expect("");

        let query = vec![0_f32, 0.99, 0.0];

        let found = cache.get(&query, 1, 0.9).expect("");
        assert_eq!(found.len(), 1);

        cache.delete(id).expect("");
        let after_delete = cache.get(&query, 1, 0.9).expect("");
        assert_eq!(after_delete.len(), 0);
    }
}

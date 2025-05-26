use mockall::automock;

use crate::cache::error::CacheError;

#[automock]
pub trait SemanticStore: Send + Sync {
    // Will return a list of sorted id's matching the query vector
    // the id's will be sorted in descending order w.r.t. similarity, most similar id first
    // similarity is [0, 1] where 0 is least similar, and 1 is most similar
    // may return fewer than top_k vectors if not enough matching the similarity threshold are found in the db
    fn get(
        &self,
        vec: &Vec<f32>,
        top_k: usize,
        similarity_threshold: f32,
    ) -> Result<Vec<u64>, CacheError>;
    fn put(&self, id: u64, vec: Vec<f32>) -> Result<(), CacheError>;
    fn delete(&self, id: u64) -> Result<(), CacheError>;
    fn memory_usage_bytes(&self) -> usize;
}

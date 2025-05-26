use mockall::automock;

use crate::cache::error::CacheError;

#[automock]
pub trait SemanticStore: Send + Sync {
    // TODO (v0): might make sense to normalize distances to be between 0 and 1 regardless of vector db impl
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

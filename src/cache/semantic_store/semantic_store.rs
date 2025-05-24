use faiss::index::SearchResult;
use mockall::automock;

use crate::cache::error::CacheError;

#[automock]
pub trait SemanticStore: Send + Sync {
    // TODO we probably want to create a repo specific SearchResult struct instead of relying on one from faiss
    // might make sense to normalize distances to be between 0 and 1 regardless of vector db impl
    fn get(&self, vec: Vec<f32>, top_k: usize) -> Result<SearchResult, CacheError>;
    fn put(&self, id: u32, vec: Vec<f32>) -> Result<(), CacheError>;
    fn delete(&self, id: u32) -> Result<(), CacheError>;
}

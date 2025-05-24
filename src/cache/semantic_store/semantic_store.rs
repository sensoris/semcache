use faiss::index::SearchResult;
use mockall::automock;

use crate::cache::error::CacheError;

#[automock]
pub(crate) trait SemanticStore: Send + Sync {
    fn get(&self, vec: Vec<f32>, top_k: usize) -> Result<SearchResult, CacheError>;
    fn put(&self, id: u32, vec: Vec<f32>) -> Result<(), CacheError>;
    fn delete(&self, id: u32) -> Result<(), CacheError>;
}

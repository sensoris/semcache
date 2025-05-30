use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum CacheError {
    #[error("Failed to search through Faiss in-memory store: {0}")]
    FaissRetrievalError(#[from] faiss::error::Error),
}

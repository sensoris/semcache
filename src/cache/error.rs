use thiserror::Error;

use crate::embedding::error::EmbeddingError;

#[derive(Error, Debug)]
pub enum CacheError {
    #[error("Failed to search through Faiss in-memory store: {0}")]
    FaissRetrievalError(#[from] faiss::error::Error),
    #[error("Failed to generate embedding: {0}")]
    CacheError(#[from] EmbeddingError),
}

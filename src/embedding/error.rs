use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum EmbeddingError {
    #[error("Failed to generate embedding: {0}")]
    GenerationError(String),
    #[error("Failed to set up embedding model: {0}")]
    SetupError(String),
}

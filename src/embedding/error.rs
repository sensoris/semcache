use thiserror::Error;

#[derive(Error, Debug)]
pub enum EmbeddingError {
    #[error("Failed to generate embedding: {0}")]
    GenerationError(String),
}

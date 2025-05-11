use crate::embedding::error::EmbeddingError;

pub trait EmbeddingService: Send + Sync {
    fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError>;
}

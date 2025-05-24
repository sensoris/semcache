use mockall::automock;

use crate::embedding::error::EmbeddingError;

#[automock]
pub trait EmbeddingService: Send + Sync {
    fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError>;
}

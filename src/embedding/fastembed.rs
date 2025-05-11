use crate::embedding::error::EmbeddingError;
use crate::embedding::service::EmbeddingService;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};

pub struct FastEmbedService {
    text_embedding: TextEmbedding,
}

impl FastEmbedService {
    pub fn new() -> Self {
        let text_embedding = TextEmbedding::try_new(
            InitOptions::new(EmbeddingModel::AllMiniLML6V2).with_show_download_progress(true),
        );

        Self {
            text_embedding: text_embedding.unwrap(),
        }
    }
}

impl EmbeddingService for FastEmbedService {
    fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        let embeddings = self
            .text_embedding
            .embed(vec![text], None)
            .map_err(|e| EmbeddingError::GenerationError(e.to_string()))?;

        // todo: is this the best way to return the vector embedding?
        Ok(embeddings[0].clone())
    }
}

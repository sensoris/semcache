use crate::embedding::error::EmbeddingError;
use crate::embedding::service::EmbeddingService;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use tracing::error;

pub struct FastEmbedService {
    text_embedding: TextEmbedding,
    model_name: EmbeddingModel,
}

impl FastEmbedService {
    pub fn new() -> Self {
        let text_embedding = TextEmbedding::try_new(
            InitOptions::new(EmbeddingModel::AllMiniLML6V2).with_show_download_progress(true),
        );

        Self {
            text_embedding: text_embedding.unwrap_or_else(|err| {
                error!(error = ?err);
                panic!("failed to init text_embedding")
            }),
            model_name: EmbeddingModel::AllMiniLML6V2,
        }
    }

    pub fn get_dimensionality(&self) -> u32 {
        match &self.model_name {
            EmbeddingModel::AllMiniLML6V2 => 384,
            _ => panic!(
                "{}",
                EmbeddingError::SetupError(String::from("Embedding model with unknown size",))
            ),
        }
    }
}

impl EmbeddingService for FastEmbedService {
    fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        let embeddings = self
            .text_embedding
            .embed(vec![text], None)
            .map_err(|e| EmbeddingError::GenerationError(e.to_string()))?;

        Ok(embeddings.into_iter().next().unwrap())
    }
}

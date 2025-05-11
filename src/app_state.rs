use crate::embedding::fastembed::FastEmbedService;
use crate::embedding::service::EmbeddingService;
use reqwest::Client;
use std::sync::Arc;

pub struct AppState {
    pub http_client: Client,
    pub embedding_service: Arc<FastEmbedService>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            http_client: Client::new(),
            embedding_service: Arc::new(FastEmbedService::new()),
        }
    }
}

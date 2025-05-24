use crate::{
    cache::{cache::Cache, semantic_store::faiss_store::FaissStore},
    embedding::fastembed::FastEmbedService,
};
use reqwest::Client;

pub struct AppState {
    pub http_client: Client,
    pub cache: Cache,
}

impl AppState {
    pub fn new() -> Self {
        let embedding_service = FastEmbedService::new();
        let semantic_store = FaissStore::new(embedding_service.get_dimensionality());
        Self {
            http_client: Client::new(),
            cache: Cache::new(Box::new(embedding_service), Box::new(semantic_store), 0.9),
        }
    }
}

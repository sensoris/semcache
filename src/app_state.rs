use crate::cache::cache::{Cache, EvictionPolicy};
use crate::cache::response_store::ResponseStore;
use crate::cache::semantic_store::flat_ip_faiss_store::FlatIPFaissStore;
use crate::embedding::fastembed::FastEmbedService;
use reqwest::Client;

pub struct AppState {
    pub http_client: Client,
    // todo we should probably use base64 or something that isn't string here
    pub cache: Cache<String>,
}

impl AppState {
    pub fn new() -> Self {
        let embedding_service = FastEmbedService::new();
        let semantic_store = FlatIPFaissStore::new(embedding_service.get_dimensionality());
        Self {
            http_client: Client::new(),
            cache: Cache::new(
                Box::new(embedding_service),
                Box::new(semantic_store),
                ResponseStore::new(),
                0.9,
                EvictionPolicy::EntryLimit(4),
            ),
        }
    }
}

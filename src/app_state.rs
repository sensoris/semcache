use crate::cache::cache::{Cache, EvictionPolicy};
use crate::cache::response_store::ResponseStore;
use crate::cache::semantic_store::flat_ip_faiss_store::FlatIPFaissStore;
use crate::clients::client::Client;
use crate::clients::http_client::HttpClient;
use crate::embedding::fastembed::FastEmbedService;
use crate::embedding::service::EmbeddingService;

pub struct AppState {
    pub http_client: Box<dyn Client>,
    pub embedding_service: Box<dyn EmbeddingService>,
    // TODO (not v0): we should probably use base64 or something that isn't string here
    pub cache: Cache<String>,
}

impl AppState {
    pub fn new() -> Self {
        let http_client = Box::new(HttpClient::new());
        let embedding_service = FastEmbedService::new();
        let semantic_store = FlatIPFaissStore::new(embedding_service.get_dimensionality());
        Self {
            http_client: http_client,
            embedding_service: Box::new(embedding_service),
            cache: Cache::new(
                Box::new(semantic_store),
                ResponseStore::new(),
                0.9,
                EvictionPolicy::EntryLimit(4),
            ),
        }
    }
}

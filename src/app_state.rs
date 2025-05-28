use crate::cache::cache::Cache;
use crate::cache::lru_cache::{EvictionPolicy, LruCache};
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
    pub cache: Box<dyn Cache<String>>,
}

impl AppState {
    pub fn new() -> Self {
        // client for upstream LLM requests
        let http_client = Box::new(HttpClient::new());
        // cache fields
        let embedding_service = Box::new(FastEmbedService::new());
        let semantic_store = Box::new(FlatIPFaissStore::new(
            embedding_service.get_dimensionality(),
        ));
        let response_store = ResponseStore::new();
        // create cache
        let cache = Box::new(LruCache::new(
            semantic_store,
            response_store,
            0.9,
            EvictionPolicy::EntryLimit(4),
        ));
        // put service dependencies into app state
        Self {
             http_client,
            embedding_service,
            cache,
        }
    }
}

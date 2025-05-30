use crate::cache::cache::Cache;
use crate::cache::cache_impl::{CacheImpl, EvictionPolicy};
use crate::cache::response_store::ResponseStore;
use crate::cache::semantic_store::flat_ip_faiss_store::FlatIPFaissStore;
use crate::clients::client::Client;
use crate::clients::http_client::HttpClient;
use crate::embedding::fastembed::FastEmbedService;
use crate::embedding::service::EmbeddingService;

pub struct AppState {
    pub http_client: Box<dyn Client>,
    pub embedding_service: Box<dyn EmbeddingService>,
    pub cache: Box<dyn Cache<String>>,
}

impl AppState {
    pub fn new(semantic_threshold: f32) -> Self {
        // client for upstream LLM requests
        let http_client = Box::new(HttpClient::new());
        // cache fields
        let embedding_service = Box::new(FastEmbedService::new());
        let semantic_store = Box::new(FlatIPFaissStore::new(
            embedding_service.get_dimensionality(),
        ));
        let response_store = ResponseStore::new();
        // create cache
        let cache = Box::new(CacheImpl::new(
            semantic_store,
            response_store,
            semantic_threshold,
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

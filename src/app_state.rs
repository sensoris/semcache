use crate::cache::cache::Cache;
use crate::cache::cache_impl::{CacheImpl, EvictionPolicy};
use crate::cache::response_store::ResponseStore;
use crate::cache::semantic_store::flat_ip_faiss_store::FlatIPFaissStore;
use crate::clients::client::Client;
use crate::clients::http_client::HttpClient;
use crate::embedding::fastembed::FastEmbedService;
use crate::embedding::service::EmbeddingService;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppStateError {
    #[error("Lock poisoned: {0}")]
    LockPoisoned(&'static str),

    #[error("Invalid namespace: {0}")]
    InvalidNamespace(&'static str),
}

pub struct AppState {
    // client for upstream LLM requests
    pub http_client: Box<dyn Client>,
    pub embedding_service: Box<dyn EmbeddingService>,
    caches: RwLock<HashMap<String, Arc<Box<dyn Cache<Vec<u8>>>>>>,
    similarity_threshold: f32,
    eviction_policy: EvictionPolicy,
}

impl AppState {
    const MAX_NAMESPACE_LENGTH: usize = 64;

    pub fn new(similarity_threshold: f32, eviction_policy: EvictionPolicy) -> Self {
        Self {
            http_client: Box::new(HttpClient::new()),
            embedding_service: Box::new(FastEmbedService::new()),
            caches: RwLock::new(HashMap::new()),
            similarity_threshold,
            eviction_policy,
        }
    }

    fn validate_namespace(namespace: &str) -> Result<(), AppStateError> {
        if namespace.is_empty() {
            return Err(AppStateError::InvalidNamespace("namespace cannot be empty"));
        }

        if namespace.len() > Self::MAX_NAMESPACE_LENGTH {
            return Err(AppStateError::InvalidNamespace("namespace too long"));
        }

        // Check for valid characters: alphanumeric, underscore, hyphen
        if !namespace
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            return Err(AppStateError::InvalidNamespace(
                "invalid characters in namespace",
            ));
        }

        Ok(())
    }

    fn create_cache(&self) -> Box<dyn Cache<Vec<u8>>> {
        let semantic_store = Box::new(FlatIPFaissStore::new(
            self.embedding_service.get_dimensionality(),
        ));
        let response_store = ResponseStore::new();
        Box::new(CacheImpl::new(
            semantic_store,
            response_store,
            self.similarity_threshold,
            self.eviction_policy,
        ))
    }

    pub fn get_cache(
        &self,
        namespace: &str,
    ) -> Result<Arc<Box<dyn Cache<Vec<u8>>>>, AppStateError> {
        Self::validate_namespace(namespace)?;

        // Try read lock first to check if cache exists
        {
            let read_guard = self
                .caches
                .read()
                .map_err(|_| AppStateError::LockPoisoned("caches read lock poisoned"))?;
            if let Some(cache) = read_guard.get(namespace) {
                return Ok(Arc::clone(cache));
            }
        }

        // Cache doesn't exist, acquire write lock to create it
        let mut write_guard = self
            .caches
            .write()
            .map_err(|_| AppStateError::LockPoisoned("caches write lock poisoned"))?;

        // Double-check: another thread might have created it while we were waiting
        let cache = write_guard
            .entry(namespace.to_string())
            .or_insert_with(|| Arc::new(self.create_cache()))
            .clone();

        Ok(cache)
    }
}

#[cfg(test)]
impl AppState {
    pub fn new_with_cache_for_test(
        http_client: Box<dyn Client>,
        embedding_service: Box<dyn EmbeddingService>,
        cache: Box<dyn Cache<Vec<u8>>>,
    ) -> Self {
        use crate::utils::header_utils::DEFAULT_NAMESPACE;
        use std::collections::HashMap;
        let mut caches = HashMap::new();
        caches.insert(DEFAULT_NAMESPACE.to_string(), Arc::new(cache));

        Self {
            http_client,
            embedding_service,
            caches: RwLock::new(caches),
            similarity_threshold: 0.9,
            eviction_policy: EvictionPolicy::EntryLimit(100),
        }
    }
}

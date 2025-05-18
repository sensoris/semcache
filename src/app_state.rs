use crate::{cache::cache::Cache, embedding::fastembed::FastEmbedService};
use reqwest::Client;
use std::sync::Arc;

pub struct AppState {
    pub http_client: Client,
    pub cache: Cache,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            http_client: Client::new(),
            cache: Cache::new(FastEmbedService::new(), 0.9),
        }
    }
}

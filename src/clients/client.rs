use async_trait::async_trait;
use axum::http::HeaderMap;
use serde_json::Value;

use crate::{endpoints::chat::error::CompletionError, providers::ProviderType};

//TODO (V0): use the test config attribute for automocks to avoid generating mock impls for non test code
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait Client: Send + Sync {
    async fn post_http_request(
        &self,
        header_map: HeaderMap,
        provider: ProviderType,
        request_body: Value,
    ) -> Result<UpstreamResponse, CompletionError>;
}

pub struct UpstreamResponse {
    pub header_map: HeaderMap,
    pub response_body: String,
}

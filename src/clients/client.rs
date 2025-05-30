use async_trait::async_trait;
use axum::http::HeaderMap;
use serde_json::Value;
use url::Url;

//TODO (V0): use the test config attribute for automocks to avoid generating mock impls for non test code
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait Client: Send + Sync {
    async fn post_http_request(
        &self,
        header_map: HeaderMap,
        upstream_url: Url,
        request_body: &Value,
    ) -> Result<reqwest::Response, reqwest::Error>;
}

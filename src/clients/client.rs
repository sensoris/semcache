use async_trait::async_trait;
use url::Url;

use crate::endpoints::chat::dto::CompletionRequest;

//TODO use the test config attribute for automocks to avoid generating mock impls for non test code
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait Client: Send + Sync {
    async fn post_http_request(
        &self,
        auth_token: &str,
        upstream_url: Url,
        request_body: &CompletionRequest,
    ) -> Result<reqwest::Response, reqwest::Error>;
}

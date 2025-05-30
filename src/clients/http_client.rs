use async_trait::async_trait;
use axum::http::HeaderMap;
use serde_json::Value;
use url::Url;

use super::client::Client;

pub struct HttpClient {
    reqwest_client: reqwest::Client,
}

#[async_trait]
impl Client for HttpClient {
    async fn post_http_request(
        &self,
        headers: HeaderMap,
        upstream_url: Url,
        request_body: &Value,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let response = self
            .reqwest_client
            .post(upstream_url)
            .headers(headers)
            .json(request_body)
            .send()
            .await?;

        Ok(response)
    }
}

impl HttpClient {
    pub fn new() -> Self {
        let reqwest_client = reqwest::Client::new();
        Self { reqwest_client }
    }
}

use async_trait::async_trait;
use axum::http::HeaderMap;
use reqwest::{Error, Response};
use serde_json::Value;

use crate::{
    providers::ProviderType,
    utils::header_utils::{prepare_upstream_headers, remove_hop_headers},
};

use super::client::{Client, UpstreamResponse};

pub struct HttpClient {
    reqwest_client: reqwest::Client,
}

#[async_trait]
impl Client for HttpClient {
    async fn post_http_request(
        &self,
        headers: HeaderMap,
        provider: ProviderType,
        request_body: Value,
    ) -> Result<UpstreamResponse, reqwest::Error> {
        // TODO (V0): look at proxy LLM upstream first
        let upstream_url = provider.url();
        let upstream_headers = prepare_upstream_headers(headers, provider);
        let reqwest_response = self
            .reqwest_client
            .post(upstream_url.clone())
            .headers(upstream_headers)
            .json(&request_body)
            .send()
            .await?;
        let response = UpstreamResponse::try_from(reqwest_response).await?;

        Ok(response)
    }
}

impl HttpClient {
    pub fn new() -> Self {
        let reqwest_client = reqwest::Client::new();
        Self { reqwest_client }
    }
}

impl UpstreamResponse {
    pub async fn try_from(reqwest_response: Response) -> Result<Self, Error> {
        let mut response_headers = reqwest_response.headers().clone();
        // todo prepare response headers
        remove_hop_headers(&mut response_headers);

        // Get the body as bytes (not JSON)
        let body_bytes = reqwest_response.bytes().await?;

        // todo, am converting into string, maybe its okay to return bytes?
        let response_string = String::from_utf8_lossy(&body_bytes).to_string();

        Ok(Self {
            header_map: response_headers,
            response_body: response_string,
        })
    }
}

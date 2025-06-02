use async_trait::async_trait;
use axum::http::HeaderMap;
use reqwest::{Error, Response};
use serde_json::Value;

use crate::{
    endpoints::chat::error::CompletionError,
    providers::ProviderType,
    utils::header_utils::{
        PROXY_UPSTREAM_HEADER, PROXY_UPSTREAM_HOST_HEADER, prepare_upstream_headers,
        remove_hop_headers,
    },
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
    ) -> Result<UpstreamResponse, CompletionError> {
        let upstream_url = provider.url(
            headers.get(&PROXY_UPSTREAM_HEADER),
            headers.get(&PROXY_UPSTREAM_HOST_HEADER),
        )?;
        let upstream_headers = prepare_upstream_headers(headers);
        let reqwest_response = self
            .reqwest_client
            .post(upstream_url)
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
    pub async fn try_from(response: Response) -> Result<Self, Error> {
        let status = response.status();

        let mut response_headers = response.headers().clone();
        remove_hop_headers(&mut response_headers);

        let body_bytes = response.bytes().await?;
        let response_string = String::from_utf8_lossy(&body_bytes).to_string();

        Ok(Self {
            status_code: status,
            header_map: response_headers,
            response_body: response_string,
        })
    }
}

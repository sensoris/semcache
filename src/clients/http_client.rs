use async_trait::async_trait;
use url::Url;

use crate::endpoints::chat::dto::CompletionRequest;

use super::client::Client;

pub struct HttpClient {
    reqwest_client: reqwest::Client,
}

#[async_trait]
impl Client for HttpClient {
    async fn send_completion_request(
        &self,
        auth_token: &str,
        upstream_url: Url,
        request_body: &CompletionRequest,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let response = self
            .reqwest_client
            .post(upstream_url)
            .header(reqwest::header::AUTHORIZATION, auth_token)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .json(&request_body)
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

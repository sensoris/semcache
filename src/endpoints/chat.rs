use axum::{
    extract::{Json, State},
    http::{self, header::HeaderMap}, response::IntoResponse,
    http::StatusCode
};
use core::panic;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

use crate::app_state::AppState;

#[derive(Deserialize, Serialize, Debug)]
pub struct CompletionRequest {
    pub model: String,
    pub messages: Vec<Message>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Message {
    pub role: String,
    pub content: String,
}



pub async fn chat_completions(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request_body): Json<CompletionRequest>,
) {
    let auth_token = extract_auth_token(&headers);
    let response = send_request(state.http_client.clone(), auth_token, &request_body).await;
    match response {
        Ok(response) => into_response(response),
        Err(error) => to_failed_response(error)
    }
}

fn extract_auth_token(headers: &HeaderMap) -> &str {
    // extract auth token
    let auth_token = match headers.get("Authorization") {
        Some(token) => token,
        None => panic!("couldnt find auth token"),
    };
    let auth_token = auth_token
        .to_str()
        .expect("auth_token was not a valid string");
    auth_token
}

async fn send_request(
    client: reqwest::Client,
    auth_token: &str,
    request_body: &CompletionRequest,
) -> Result<reqwest::Response, reqwest::Error> {
    let response = client
        .post("https://api.openai.com/chat/completions")
        .bearer_auth(auth_token)
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await?;
    Ok(response)
}

async fn into_response(response: reqwest::Response) -> impl IntoResponse {
    let response_body = response.json<Value>().await?;
    (StatusCode::OK, response_body)
}

use axum::{
    extract::{Json, State},
    http::header::HeaderMap,
};
use std::sync::Arc;

use crate::app_state::AppState;

use super::{
    dto::{CompletionRequest, CompletionResponse},
    errors::CompletionError,
};

pub async fn completions(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request_body): Json<CompletionRequest>,
) -> Result<CompletionResponse, CompletionError> {
    let auth_token = extract_auth_token(&headers)?;
    let reqwest_response =
        send_request(state, auth_token, &request_body).await?;
    let response = CompletionResponse::from_reqwest(reqwest_response).await?;
    Ok(response)
}

fn extract_auth_token(headers: &HeaderMap) -> Result<&str, CompletionError> {
    // extract auth token
    headers
        .get(axum::http::header::AUTHORIZATION)
        .ok_or(CompletionError::InputValidation(String::from(
            "Missing authorization header",
        )))?
        .to_str()
        .map_err(|error| {
            CompletionError::InputValidation(format!(
                "authorization header could not be parsed as a string, {}",
                error
            ))
        })
}

async fn send_request(
    state: Arc<AppState>,
    auth_token: &str,
    request_body: &CompletionRequest,
) -> Result<reqwest::Response, reqwest::Error> {
    let response = state.http_client
        .post("https://api.openai.com/v1/chat/completions")
        .header(reqwest::header::AUTHORIZATION, auth_token)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .json(&request_body)
        .send()
        .await?;
    Ok(response)
}

use axum::{
    extract::{Json, State},
    http::StatusCode,
    http::header::HeaderMap,
    response::IntoResponse,
    response::Response,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

use crate::app_state::AppState;

// DTO's

// input type
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

// Response type
pub struct CompletionResponse {
    body: Json<Value>,
}

impl CompletionResponse {
    pub async fn from_reqwest(res: reqwest::Response) -> Result<Self, CompletionError> {
        let status = res.status();
        let text = res.text().await?;

        // You can log it here
        eprintln!("Upstream returned status {} with body:\n{}", status, text);
        let parsed: serde_json::Value = serde_json::from_str(&text)?;
        Ok(Self { body: Json(parsed) })
    }
}

impl IntoResponse for CompletionResponse {
    fn into_response(self) -> Response {
        (StatusCode::OK, self.body).into_response()
    }
}

// Error type
pub enum CompletionError {
    Upstream(reqwest::Error),
    InvalidJson(serde_json::Error),
    InputValidation(String),
}

impl From<reqwest::Error> for CompletionError {
    fn from(err: reqwest::Error) -> Self {
        CompletionError::Upstream(err)
    }
}

impl From<serde_json::Error> for CompletionError {
    fn from(err: serde_json::Error) -> Self {
        CompletionError::InvalidJson(err)
    }
}

impl IntoResponse for CompletionError {
    fn into_response(self) -> Response {
        match self {
            Self::Upstream(reqwest_err) => {
                eprintln!(
                    "Error: {} when calling upstream: {}, with status code: {}",
                    reqwest_err.to_string(),
                    reqwest_err
                        .url()
                        .map(|url| url.as_str())
                        .get_or_insert("NO_UPSTREAM"),
                    reqwest_err
                        .status()
                        .get_or_insert(reqwest::StatusCode::INTERNAL_SERVER_ERROR)
                );
                (StatusCode::INTERNAL_SERVER_ERROR, "Failed to call upstream").into_response()
            }
            Self::InvalidJson(serde_error) => {
                eprintln!("error parsing json {}", serde_error);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to parse json from upstream",
                )
                    .into_response()
            }
            Self::InputValidation(message) => {
                eprint!("Failed to parse input, {}", message);
                (StatusCode::BAD_REQUEST, message).into_response()
            }
        }
    }
}

pub async fn chat_completions(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request_body): Json<CompletionRequest>,
) -> Result<CompletionResponse, CompletionError> {
    let auth_token = extract_auth_token(&headers)?;
    let reqwest_response =
        send_request(state.http_client.clone(), auth_token, &request_body).await?;
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
    client: reqwest::Client,
    auth_token: &str,
    request_body: &CompletionRequest,
) -> Result<reqwest::Response, reqwest::Error> {
    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header(reqwest::header::AUTHORIZATION, auth_token)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .json(&request_body)
        .send()
        .await?;
    Ok(response)
}

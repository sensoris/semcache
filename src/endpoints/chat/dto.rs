use axum::{Json, response::IntoResponse, response::Response};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::error::CompletionError;

// DTO's for the chat handler

// Request type
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
#[derive(Clone)]
pub struct CompletionResponse {
    body: Json<Value>,
}

impl TryFrom<&CompletionResponse> for String {
    type Error = serde_json::Error;

    fn try_from(resp: &CompletionResponse) -> Result<Self, Self::Error> {
        serde_json::to_string(&resp.body.0)
    }
}

impl CompletionResponse {
    pub async fn from_reqwest(res: reqwest::Response) -> Result<Self, CompletionError> {
        let body = res.json::<Value>().await?;
        Ok(Self { body: Json(body) })
    }

    pub fn from_cache(saved_response: String) -> Result<Self, serde_json::Error> {
        let parsed = serde_json::from_str(&saved_response)?;
        Ok(Self {
            body: Json(parsed),
        })
    }
}

impl IntoResponse for CompletionResponse {
    fn into_response(self) -> Response {
        (StatusCode::OK, self.body).into_response()
    }
}

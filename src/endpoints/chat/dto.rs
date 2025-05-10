use axum::{Json, response::IntoResponse, response::Response};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::CompletionError;

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
pub struct CompletionResponse {
    body: Json<Value>,
}

impl CompletionResponse {
    pub async fn from_reqwest(res: reqwest::Response) -> Result<Self, CompletionError> {
        let body = res.json::<Value>().await?;
        Ok(Self { body: Json(body) })
    }
}

impl IntoResponse for CompletionResponse {
    fn into_response(self) -> Response {
        (StatusCode::OK, self.body).into_response()
    }
}

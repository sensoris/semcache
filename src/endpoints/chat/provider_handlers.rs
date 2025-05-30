use crate::app_state::AppState;
use crate::endpoints::chat::error::CompletionError;
use crate::endpoints::chat::handler::completions;
use crate::providers::{anthropic::Anthropic, generic::Generic, openai::OpenAI};
use axum::{
    Json,
    extract::State,
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use serde_json::Value;
use std::sync::Arc;

pub async fn openai_handler(
    state: State<Arc<AppState>>,
    headers: HeaderMap,
    body: Json<Value>,
) -> Result<Response, CompletionError> {
    completions(state, headers, body, OpenAI)
        .await
        .map(IntoResponse::into_response)
}

pub async fn anthropic_handler(
    state: State<Arc<AppState>>,
    headers: HeaderMap,
    body: Json<Value>,
) -> Result<Response, CompletionError> {
    completions(state, headers, body, Anthropic)
        .await
        .map(IntoResponse::into_response)
}

pub async fn generic_handler(
    state: State<Arc<AppState>>,
    headers: HeaderMap,
    body: Json<Value>,
) -> Result<Response, CompletionError> {
    completions(state, headers, body, Generic)
        .await
        .map(IntoResponse::into_response)
}

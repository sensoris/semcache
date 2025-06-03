use crate::endpoints::chat::error::CompletionError;
use crate::endpoints::chat::handler::completions;
use crate::{app_state::AppState, providers::ProviderType};
use axum::{Json, extract::State, http::HeaderMap, response::Response};
use serde_json::Value;
use std::sync::Arc;

pub async fn openai_handler(
    state: State<Arc<AppState>>,
    headers: HeaderMap,
    body: Json<Value>,
) -> Result<Response, CompletionError> {
    completions(state, headers, body, ProviderType::OpenAI).await
}

pub async fn anthropic_handler(
    state: State<Arc<AppState>>,
    headers: HeaderMap,
    body: Json<Value>,
) -> Result<Response, CompletionError> {
    completions(state, headers, body, ProviderType::Anthropic).await
}

pub async fn generic_handler(
    state: State<Arc<AppState>>,
    headers: HeaderMap,
    body: Json<Value>,
) -> Result<Response, CompletionError> {
    completions(state, headers, body, ProviderType::Generic).await
}

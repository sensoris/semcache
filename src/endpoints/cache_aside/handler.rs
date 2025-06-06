use axum::response::{IntoResponse, Response};
use std::sync::Arc;

use axum::{Json, extract::State, http::HeaderMap};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{app_state::AppState, embedding::error::EmbeddingError};

#[derive(Debug, Error)]
pub enum CacheAsideError {
    #[error("Upstream request failed: {0}")]
    InternalEmbeddingError(#[from] EmbeddingError),
}

impl IntoResponse for CacheAsideError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR).into_response()
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GetRequest {
    pub query: String,
}

pub struct GetResponse {}

#[derive(Deserialize, Serialize, Debug)]
pub struct PutRequest {
    pub query: String,
    pub body: String,
}

pub async fn get(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request_body): Json<GetRequest>,
) -> Result<Response, CacheAsideError> {
    unimplemented!("unimplemented")
}

pub async fn put(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request_body): Json<PutRequest>,
) -> Result<Response, CacheAsideError> {
    unimplemented!("unimplemented")
}


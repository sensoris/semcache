use axum::{
    http::HeaderValue,
    response::{IntoResponse, Response},
};
use std::sync::Arc;
use tracing::error;

use axum::{Json, extract::State, http::HeaderMap};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{app_state::AppState, cache::error::CacheError, embedding::error::EmbeddingError};

#[derive(Debug, Error)]
pub enum CacheAsideError {
    #[error("Failed to generate embedding: {0}")]
    InternalEmbedding(#[from] EmbeddingError),
    #[error("Error in caching layer: {0}")]
    InternalCache(#[from] CacheError),
    #[error("Invalid input: {0}")]
    InputValidation(String),
}

impl IntoResponse for CacheAsideError {
    fn into_response(self) -> Response {
        match self {
            Self::InternalEmbedding(err) => {
                error!(?err, "returning internal error to user");
                (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong").into_response()
            }
            Self::InternalCache(err) => {
                error!(?err, "returning internal error to user");
                (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong").into_response()
            }
            Self::InputValidation(err) => {
                (StatusCode::BAD_REQUEST, err.to_string()).into_response()
            }
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GetRequest {
    pub query: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PutRequest {
    pub query: String,
    pub body: String,
}

pub async fn get(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<GetRequest>,
) -> Result<Response, CacheAsideError> {
    validate_headers(&headers)?;

    let embedding = state.embedding_service.embed(&request.query)?;
    let saved_response = state.cache.get_if_present(&embedding)?;
    let http_response = match saved_response {
        Some(response_bytes) => (StatusCode::OK, response_bytes).into_response(),
        None => (StatusCode::NOT_FOUND).into_response(),
    };
    Ok(http_response)
}

pub async fn put(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<PutRequest>,
) -> Result<Response, CacheAsideError> {
    validate_headers(&headers)?;

    let body: Vec<u8> = request.body.into_bytes();
    let embedding = state.embedding_service.embed(&request.query)?;
    // if we already have an entry associated with the prompt, update it
    let updated_existing_entry = state.cache.try_update(&embedding, body.clone())?;
    if !updated_existing_entry {
        state.cache.insert(embedding, body)?;
    }
    Ok((StatusCode::OK).into_response())
}

fn validate_headers(headers: &HeaderMap) -> Result<(), CacheAsideError> {
    if headers.get("Accept") != Some(&HeaderValue::from_static("application/json")) {
        return Err(CacheAsideError::InputValidation(String::from(
            "The cache aside endpoint only supports application/json at this stage",
        )));
    }
    Ok(())
}

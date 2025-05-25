use axum::response::{IntoResponse, Response};
use reqwest::StatusCode;

use thiserror::Error;
use tracing::warn;

use crate::{cache::error::CacheError, embedding::error::EmbeddingError};

// Error type
#[derive(Debug, Error)]
pub enum CompletionError {
    #[error("Upstream request failed: {0}")]
    Upstream(#[from] reqwest::Error),

    #[error("Invalid JSON: {0}")]
    InvalidResponse(#[from] serde_json::Error),

    #[error("Input validation error: {0}")]
    InvalidRequest(String),

    #[error("Error in caching layer: {0}")]
    InternalCacheError(#[from] CacheError),

    #[error("Error generating embedding: {0}")]
    InternalEmbeddingError(#[from] EmbeddingError),
}

impl IntoResponse for CompletionError {
    fn into_response(self) -> Response {
        match self {
            Self::Upstream(reqwest_err) => {
                warn!(
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
            Self::InvalidResponse(serde_error) => {
                warn!("error parsing json {}", serde_error);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to parse json from upstream",
                )
                    .into_response()
            }
            Self::InvalidRequest(message) => {
                warn!("Failed to parse input, {}", message);
                (StatusCode::BAD_REQUEST, message).into_response()
            }
            Self::InternalCacheError(internal_errror) => {
                warn!("Internal caching error: {}", internal_errror);
                (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong!").into_response()
            }
            Self::InternalEmbeddingError(internal_error) => {
                warn!("Internal embedding error: {}", internal_error);
                (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong!").into_response()
            }
        }
    }
}

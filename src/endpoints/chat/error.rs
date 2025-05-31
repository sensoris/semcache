use axum::response::{IntoResponse, Response};
use jsonpath_rust::parser::errors::JsonPathError;
use reqwest::StatusCode;

use thiserror::Error;
use tracing::warn;

use crate::{cache::error::CacheError, embedding::error::EmbeddingError, providers::ProviderError};

// Error type
#[derive(Debug, Error)]
pub enum CompletionError {
    #[error("Upstream request failed: {0}")]
    Upstream(#[from] reqwest::Error),

    #[error("Invalid JSON: {0}")]
    InvalidResponse(#[from] serde_json::Error),

    #[error("Input validation error: {0}")]
    InvalidRequest(String),

    // todo is there a way to combine this with the above invalidRequest?
    #[error("Input validation error: {0}")]
    InvalidJsonPath(#[from] JsonPathError),

    #[error("Error in caching layer: {0}")]
    InternalCacheError(#[from] CacheError),

    #[error("Error generating embedding: {0}")]
    InternalEmbeddingError(#[from] EmbeddingError),
    #[error("Provider error: {0}")]
    InternalProviderError(#[from] ProviderError),
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
            Self::InternalCacheError(internal_error) => {
                warn!("Internal caching error: {}", internal_error);
                (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong!").into_response()
            }
            Self::InternalEmbeddingError(internal_error) => {
                warn!("Internal embedding error: {}", internal_error);
                (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong!").into_response()
            }
            Self::InvalidJsonPath(message) => {
                warn!("Failed to parse input, {}", message);
                (StatusCode::BAD_REQUEST, message.to_string()).into_response()
            }
            Self::InternalProviderError(err) => {
                warn!("Error in provider: {}", err);
                match err {
                    ProviderError::HeaderParsingError(to_str_error) => (
                        StatusCode::BAD_REQUEST,
                        format!("Error parsing headers: {to_str_error}"),
                    )
                        .into_response(),
                }
            }
        }
    }
}

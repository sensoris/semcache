use axum::response::{IntoResponse, Response};
use std::sync::Arc;
use tracing::{debug, error};

use axum::{Json, extract::State};
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
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GetRequest {
    pub key: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PutRequest {
    pub key: String,
    pub data: String,
}

pub async fn get(
    State(state): State<Arc<AppState>>,
    Json(request): Json<GetRequest>,
) -> Result<Response, CacheAsideError> {
    debug!("cache_aside::GET request received");
    let embedding = state.embedding_service.embed(&request.key)?;
    let saved_response = state.cache.get_if_present(&embedding)?;
    let http_response = match saved_response {
        Some(response_bytes) => (StatusCode::OK, response_bytes).into_response(),
        None => (StatusCode::NOT_FOUND).into_response(),
    };
    Ok(http_response)
}

pub async fn put(
    State(state): State<Arc<AppState>>,
    Json(request): Json<PutRequest>,
) -> Result<Response, CacheAsideError> {
    debug!("cache_aside::PUT request received");
    let body: Vec<u8> = request.data.into_bytes();
    let embedding = state.embedding_service.embed(&request.key)?;
    // if we already have an entry associated with the prompt, update it
    let updated_existing_entry = state.cache.try_update(&embedding, body.clone())?;
    if !updated_existing_entry {
        state.cache.insert(embedding, body)?;
    }
    Ok((StatusCode::OK).into_response())
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, usize};

    use axum::{body, extract::State};
    use mockall::predicate::eq;
    use reqwest::StatusCode;

    use crate::{
        app_state::AppState,
        cache::{cache::MockCache, error::CacheError},
        embedding::{error::EmbeddingError, service::MockEmbeddingService},
        endpoints::cache_aside::handler::{CacheAsideError, GetRequest, PutRequest, get, put},
    };

    #[tokio::test]
    async fn get_should_return_error_on_cache_failure() {
        // given
        let prompt = "test prompt";
        let embedding = vec![0.1, 0.2, 0.3];

        // set up embedding service mock
        let mut mock_embed = MockEmbeddingService::new();
        mock_embed
            .expect_embed()
            .times(1)
            .returning(move |_| Ok(embedding.clone()));

        // set up cache mock
        let mut mock_cache: MockCache<Vec<u8>> = MockCache::new();
        mock_cache.expect_get_if_present().returning(|_| {
            Err(CacheError::FaissRetrievalError(
                faiss::error::Error::IndexDescription,
            ))
        });

        // set up client mock and assert we don't reach it
        let mut mock_client = crate::clients::client::MockClient::new();
        mock_client.expect_post_http_request().times(0);

        // put mocked objects into the appstate
        let app_state = Arc::new(AppState {
            embedding_service: Box::new(mock_embed),
            cache: Box::new(mock_cache),
            http_client: Box::new(mock_client),
        });

        let request_body = GetRequest {
            key: String::from(prompt),
        };

        // when
        let result = get(State(app_state), axum::Json(request_body)).await;

        // then
        match result {
            Err(CacheAsideError::InternalCache(_)) => {}
            _ => panic!("Expected CacheAsideError::InternalCache"),
        }
    }

    #[tokio::test]
    async fn get_should_return_error_on_embedding_failure() {
        // given
        let prompt = "test prompt";

        // set up embedding service mock
        let mut mock_embed = MockEmbeddingService::new();
        mock_embed.expect_embed().times(1).returning(move |_| {
            Err(EmbeddingError::GenerationError(String::from(
                "failed to generate embedding",
            )))
        });

        // set up cache mock
        let mut mock_cache = MockCache::new();
        mock_cache.expect_get_if_present().times(0);

        // set up client mock and assert we don't reach it
        let mut mock_client = crate::clients::client::MockClient::new();
        mock_client.expect_post_http_request().times(0);

        // put mocked objects into the appstate
        let app_state = Arc::new(AppState {
            embedding_service: Box::new(mock_embed),
            cache: Box::new(mock_cache),
            http_client: Box::new(mock_client),
        });

        let request_body = GetRequest {
            key: String::from(prompt),
        };

        // when
        let result = get(State(app_state), axum::Json(request_body)).await;

        // then
        match result {
            Err(CacheAsideError::InternalEmbedding(_)) => {}
            _ => panic!("Expected CacheAsideError::InternalEmbedding"),
        }
    }

    #[tokio::test]
    async fn get_should_return_cached_body_if_present() {
        // given
        let prompt = "test prompt";
        let embedding = vec![0.1, 0.2, 0.3];
        let response = "A".repeat(100).into_bytes();

        // set up embedding service mock
        let mut mock_embed = MockEmbeddingService::new();
        mock_embed.expect_embed().times(1).returning({
            let embedding_clone = embedding.clone();
            move |_| Ok(embedding_clone.clone())
        });

        // set up cache mock
        let mut mock_cache: MockCache<Vec<u8>> = MockCache::new();
        mock_cache
            .expect_get_if_present()
            .with(eq(embedding))
            .returning({
                let response_clone = response.clone();
                move |_| Ok(Some(response_clone.clone()))
            });

        // set up client mock and assert we don't reach it
        let mut mock_client = crate::clients::client::MockClient::new();
        mock_client.expect_post_http_request().times(0);

        // put mocked objects into the appstate
        let app_state = Arc::new(AppState {
            embedding_service: Box::new(mock_embed),
            cache: Box::new(mock_cache),
            http_client: Box::new(mock_client),
        });

        let request_body = GetRequest {
            key: String::from(prompt),
        };

        // when
        let result = get(State(app_state), axum::Json(request_body))
            .await
            .unwrap();
        let response_bytes = body::to_bytes(result.into_body(), usize::MAX)
            .await
            .unwrap();

        // then
        assert_eq!(response, response_bytes);
    }

    #[tokio::test]
    async fn get_should_return_not_found_if_cache_empty() {
        // given
        let prompt = "test prompt";
        let embedding = vec![0.1, 0.2, 0.3];

        // set up embedding service mock
        let mut mock_embed = MockEmbeddingService::new();
        mock_embed.expect_embed().times(1).returning({
            let embedding_clone = embedding.clone();
            move |_| Ok(embedding_clone.clone())
        });

        // set up cache mock
        let mut mock_cache: MockCache<Vec<u8>> = MockCache::new();
        mock_cache
            .expect_get_if_present()
            .with(eq(embedding))
            .returning(move |_| Ok(None));

        // set up client mock and assert we don't reach it
        let mut mock_client = crate::clients::client::MockClient::new();
        mock_client.expect_post_http_request().times(0);

        // put mocked objects into the appstate
        let app_state = Arc::new(AppState {
            embedding_service: Box::new(mock_embed),
            cache: Box::new(mock_cache),
            http_client: Box::new(mock_client),
        });

        let request_body = GetRequest {
            key: String::from(prompt),
        };

        // when
        let result = get(State(app_state), axum::Json(request_body))
            .await
            .unwrap();

        // then
        assert_eq!(StatusCode::NOT_FOUND, result.status());
    }

    #[tokio::test]
    async fn put_should_return_error_on_cache_failure() {
        // given
        let prompt = "test prompt";
        let body = "body ody";
        let embedding = vec![0.1, 0.2, 0.3];

        // set up embedding service mock
        let mut mock_embed = MockEmbeddingService::new();
        mock_embed
            .expect_embed()
            .times(1)
            .returning(move |_| Ok(embedding.clone()));

        // set up cache mock
        let mut mock_cache: MockCache<Vec<u8>> = MockCache::new();
        mock_cache.expect_try_update().returning(|_, _| {
            Err(CacheError::FaissRetrievalError(
                faiss::error::Error::IndexDescription,
            ))
        });

        // set up client mock and assert we don't reach it
        let mut mock_client = crate::clients::client::MockClient::new();
        mock_client.expect_post_http_request().times(0);

        // put mocked objects into the appstate
        let app_state = Arc::new(AppState {
            embedding_service: Box::new(mock_embed),
            cache: Box::new(mock_cache),
            http_client: Box::new(mock_client),
        });

        let request_body = PutRequest {
            key: String::from(prompt),
            data: String::from(body),
        };

        // when
        let result = put(State(app_state), axum::Json(request_body)).await;

        // then
        match result {
            Err(CacheAsideError::InternalCache(_)) => {}
            _ => panic!("Expected CacheAsideError::InternalCache"),
        }
    }

    #[tokio::test]
    async fn put_should_return_error_on_embedding_failure() {
        // given
        let prompt = "test prompt";
        let body = "body ody";

        // set up embedding service mock
        let mut mock_embed = MockEmbeddingService::new();
        mock_embed.expect_embed().times(1).returning(move |_| {
            Err(EmbeddingError::GenerationError(String::from(
                "failed to generate embedding",
            )))
        });

        // set up cache mock
        let mut mock_cache = MockCache::new();
        mock_cache.expect_try_update().times(0);
        mock_cache.expect_insert().times(0);

        // set up client mock and assert we don't reach it
        let mut mock_client = crate::clients::client::MockClient::new();
        mock_client.expect_post_http_request().times(0);

        // put mocked objects into the appstate
        let app_state = Arc::new(AppState {
            embedding_service: Box::new(mock_embed),
            cache: Box::new(mock_cache),
            http_client: Box::new(mock_client),
        });

        let request_body = PutRequest {
            key: String::from(prompt),
            data: String::from(body),
        };

        // when
        let result = put(State(app_state), axum::Json(request_body)).await;

        // then
        match result {
            Err(CacheAsideError::InternalEmbedding(_)) => {}
            _ => panic!("Expected CacheAsideError::InternalEmbedding"),
        }
    }

    #[tokio::test]
    async fn put_should_overwrite_if_it_exists() {
        // given
        let prompt = "test prompt";
        let body = "body ody";
        let embedding = vec![0.1, 0.2, 0.3];

        // set up embedding service mock
        let mut mock_embed = MockEmbeddingService::new();
        mock_embed
            .expect_embed()
            .times(1)
            .returning(move |_| Ok(embedding.clone()));

        // set up cache mock
        let mut mock_cache: MockCache<Vec<u8>> = MockCache::new();
        mock_cache.expect_try_update().returning(|_, _| Ok(true));

        // set up client mock and assert we don't reach it
        let mut mock_client = crate::clients::client::MockClient::new();
        mock_client.expect_post_http_request().times(0);

        // put mocked objects into the appstate
        let app_state = Arc::new(AppState {
            embedding_service: Box::new(mock_embed),
            cache: Box::new(mock_cache),
            http_client: Box::new(mock_client),
        });

        let request_body = PutRequest {
            key: String::from(prompt),
            data: String::from(body),
        };

        // when
        let result = put(State(app_state), axum::Json(request_body))
            .await
            .unwrap();

        // then
        assert_eq!(result.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn put_should_insert_if_doesnt_exist() {
        // given
        let prompt = "test prompt";
        let data = String::from("body ody");
        let embedding = vec![0.1, 0.2, 0.3];

        // set up embedding service mock
        let mut mock_embed = MockEmbeddingService::new();
        mock_embed.expect_embed().times(1).returning({
            let embedding_clone = embedding.clone();
            move |_| Ok(embedding_clone.clone())
        });

        // set up cache mock
        let mut mock_cache: MockCache<Vec<u8>> = MockCache::new();
        mock_cache
            .expect_try_update()
            .times(1)
            .with(eq(embedding.clone()), eq(data.clone().into_bytes()))
            .returning(|_, _| Ok(false));
        mock_cache
            .expect_insert()
            .times(1)
            .with(eq(embedding.clone()), eq(data.clone().into_bytes()))
            .returning(|_, _| Ok(()));

        // set up client mock and assert we don't reach it
        let mut mock_client = crate::clients::client::MockClient::new();
        mock_client.expect_post_http_request().times(0);

        // put mocked objects into the appstate
        let app_state = Arc::new(AppState {
            embedding_service: Box::new(mock_embed),
            cache: Box::new(mock_cache),
            http_client: Box::new(mock_client),
        });

        let request_body = PutRequest {
            key: String::from(prompt),
            data,
        };

        // when
        let result = put(State(app_state), axum::Json(request_body))
            .await
            .unwrap();

        // then
        assert_eq!(result.status(), StatusCode::OK);
    }
}

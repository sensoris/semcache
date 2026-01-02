use axum::response::{IntoResponse, Response};
use std::sync::Arc;
use tracing::{debug, error};

use axum::{Json, extract::State};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    app_state::{AppState, AppStateError},
    cache::error::CacheError,
    embedding::error::EmbeddingError,
};

#[derive(Debug, Error)]
pub enum CacheAsideError {
    #[error("Failed to generate embedding: {0}")]
    InternalEmbedding(#[from] EmbeddingError),
    #[error("Error in caching layer: {0}")]
    InternalCache(#[from] CacheError),
    #[error("AppState error: {0}")]
    AppStateError(#[from] AppStateError),
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
            Self::AppStateError(err) => {
                error!(?err, "AppState error");
                match err {
                    AppStateError::InvalidNamespace(_) => {
                        (StatusCode::BAD_REQUEST, err.to_string()).into_response()
                    }
                    _ => {
                        (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong").into_response()
                    }
                }
            }
        }
    }
}

use crate::utils::header_utils::DEFAULT_NAMESPACE;

fn default_namespace() -> String {
    DEFAULT_NAMESPACE.to_string()
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GetRequest {
    pub key: String,
    #[serde(default = "default_namespace")]
    pub namespace: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PutRequest {
    pub key: String,
    pub data: String,
    #[serde(default = "default_namespace")]
    pub namespace: String,
}

pub async fn get(
    State(state): State<Arc<AppState>>,
    Json(request): Json<GetRequest>,
) -> Result<Response, CacheAsideError> {
    debug!(
        "cache_aside::GET request received for namespace: {}",
        request.namespace
    );

    // Get cache for namespace
    let cache = state.get_cache(&request.namespace)?;

    let embedding = state.embedding_service.embed(&request.key)?;
    let saved_response = cache.get_if_present(&embedding)?;
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
    debug!(
        "cache_aside::PUT request received for namespace: {}",
        request.namespace
    );

    // Get cache for namespace
    let cache = state.get_cache(&request.namespace)?;

    let body: Vec<u8> = request.data.into_bytes();
    let embedding = state.embedding_service.embed(&request.key)?;
    // if we already have an entry associated with the prompt, update it
    let updated_existing_entry = cache.try_update(&embedding, body.clone())?;
    if !updated_existing_entry {
        cache.insert(embedding, body)?;
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
        let app_state = Arc::new(AppState::new_with_cache_for_test(
            Box::new(mock_client),
            Box::new(mock_embed),
            Box::new(mock_cache),
        ));

        let request_body = GetRequest {
            key: String::from(prompt),
            namespace: "default".to_string(),
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
        let app_state = Arc::new(AppState::new_with_cache_for_test(
            Box::new(mock_client),
            Box::new(mock_embed),
            Box::new(mock_cache),
        ));

        let request_body = GetRequest {
            key: String::from(prompt),
            namespace: "default".to_string(),
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
        let app_state = Arc::new(AppState::new_with_cache_for_test(
            Box::new(mock_client),
            Box::new(mock_embed),
            Box::new(mock_cache),
        ));

        let request_body = GetRequest {
            key: String::from(prompt),
            namespace: "default".to_string(),
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
        let app_state = Arc::new(AppState::new_with_cache_for_test(
            Box::new(mock_client),
            Box::new(mock_embed),
            Box::new(mock_cache),
        ));

        let request_body = GetRequest {
            key: String::from(prompt),
            namespace: "default".to_string(),
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
        let app_state = Arc::new(AppState::new_with_cache_for_test(
            Box::new(mock_client),
            Box::new(mock_embed),
            Box::new(mock_cache),
        ));

        let request_body = PutRequest {
            key: String::from(prompt),
            data: String::from(body),
            namespace: "default".to_string(),
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
        let app_state = Arc::new(AppState::new_with_cache_for_test(
            Box::new(mock_client),
            Box::new(mock_embed),
            Box::new(mock_cache),
        ));

        let request_body = PutRequest {
            key: String::from(prompt),
            data: String::from(body),
            namespace: "default".to_string(),
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
        let app_state = Arc::new(AppState::new_with_cache_for_test(
            Box::new(mock_client),
            Box::new(mock_embed),
            Box::new(mock_cache),
        ));

        let request_body = PutRequest {
            key: String::from(prompt),
            data: String::from(body),
            namespace: "default".to_string(),
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
        let app_state = Arc::new(AppState::new_with_cache_for_test(
            Box::new(mock_client),
            Box::new(mock_embed),
            Box::new(mock_cache),
        ));

        let request_body = PutRequest {
            key: String::from(prompt),
            data,
            namespace: "default".to_string(),
        };

        // when
        let result = put(State(app_state), axum::Json(request_body))
            .await
            .unwrap();

        // then
        assert_eq!(result.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn namespace_isolation_put_and_get() {
        use crate::cache::cache_impl::EvictionPolicy;

        // Create real AppState (not mocked) to test namespace isolation
        let app_state = Arc::new(AppState::new(0.9, EvictionPolicy::EntryLimit(100)));

        let key = "What is the capital of France?";

        // Put different values in different namespaces
        let put_ns1 = PutRequest {
            key: key.to_string(),
            data: "Paris for namespace 1".to_string(),
            namespace: "namespace-1".to_string(),
        };
        put(State(app_state.clone()), axum::Json(put_ns1))
            .await
            .unwrap();

        let put_ns2 = PutRequest {
            key: key.to_string(),
            data: "Paris for namespace 2".to_string(),
            namespace: "namespace-2".to_string(),
        };
        put(State(app_state.clone()), axum::Json(put_ns2))
            .await
            .unwrap();

        // Get from namespace-1 should return namespace-1's value
        let get_ns1 = GetRequest {
            key: key.to_string(),
            namespace: "namespace-1".to_string(),
        };
        let result_ns1 = get(State(app_state.clone()), axum::Json(get_ns1))
            .await
            .unwrap();
        assert_eq!(result_ns1.status(), StatusCode::OK);
        let body_ns1 = axum::body::to_bytes(result_ns1.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(body_ns1, "Paris for namespace 1".as_bytes());

        // Get from namespace-2 should return namespace-2's value
        let get_ns2 = GetRequest {
            key: key.to_string(),
            namespace: "namespace-2".to_string(),
        };
        let result_ns2 = get(State(app_state.clone()), axum::Json(get_ns2))
            .await
            .unwrap();
        assert_eq!(result_ns2.status(), StatusCode::OK);
        let body_ns2 = axum::body::to_bytes(result_ns2.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(body_ns2, "Paris for namespace 2".as_bytes());
    }

    #[tokio::test]
    async fn namespace_not_found_in_different_namespace() {
        use crate::cache::cache_impl::EvictionPolicy;

        let app_state = Arc::new(AppState::new(0.9, EvictionPolicy::EntryLimit(100)));

        // Put in namespace-1
        let put_req = PutRequest {
            key: "test key".to_string(),
            data: "test value".to_string(),
            namespace: "namespace-1".to_string(),
        };
        put(State(app_state.clone()), axum::Json(put_req))
            .await
            .unwrap();

        // Try to get from namespace-2 (should not find it)
        let get_req = GetRequest {
            key: "test key".to_string(),
            namespace: "namespace-2".to_string(),
        };
        let result = get(State(app_state), axum::Json(get_req)).await.unwrap();
        assert_eq!(result.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn invalid_namespace_returns_error() {
        use crate::cache::cache_impl::EvictionPolicy;

        let app_state = Arc::new(AppState::new(0.9, EvictionPolicy::EntryLimit(100)));

        // Empty namespace
        let get_req = GetRequest {
            key: "test".to_string(),
            namespace: "".to_string(),
        };
        let result = get(State(app_state.clone()), axum::Json(get_req)).await;
        assert!(result.is_err());

        // Invalid characters
        let get_req = GetRequest {
            key: "test".to_string(),
            namespace: "test@namespace".to_string(),
        };
        let result = get(State(app_state.clone()), axum::Json(get_req)).await;
        assert!(result.is_err());

        // Too long
        let get_req = GetRequest {
            key: "test".to_string(),
            namespace: "a".repeat(65),
        };
        let result = get(State(app_state), axum::Json(get_req)).await;
        assert!(result.is_err());
    }
}

use axum::response::{IntoResponse, Response};

use axum::{
    extract::{Json, State},
    http::{StatusCode, header::HeaderMap},
};
use serde_json::Value;
use std::sync::Arc;
use tracing::debug;

use super::error::CompletionError;
use crate::app_state::AppState;
use crate::metrics::metrics::{CACHE_HIT, CACHE_MISS, CacheStatus};
use crate::providers::ProviderType;
use crate::utils::{
    header_utils::PROXY_PROMPT_LOCATION_HEADER, json_extract::extract_prompt_from_path,
};

pub async fn completions(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request_body): Json<Value>,
    provider: ProviderType,
) -> Result<Response, CompletionError> {
    let prompt = extract_prompt_from_path(
        &request_body,
        provider.prompt_json_path(headers.get(&PROXY_PROMPT_LOCATION_HEADER))?,
    )?;
    let embedding = state.embedding_service.embed(&prompt)?;

    if let Some(saved_response) = state.cache.get_if_present(&embedding)? {
        // Return cached response with 200 OK and minimal headers
        let mut response_headers = HeaderMap::new();
        response_headers.insert("X-Cache-Status", "hit".parse().unwrap());
        response_headers.insert("content-type", "application/json".parse().unwrap());

        let mut response = (StatusCode::OK, response_headers, saved_response).into_response();

        debug!("Cache hit - returning cached response");
        CACHE_HIT.inc();
        response.extensions_mut().insert(CacheStatus::Hit);

        return Ok(response);
    };

    let upstream_response = state
        .http_client
        .post_http_request(headers, provider, request_body)
        .await?;

    // only store the response if the status code of the response is 2XX
    if upstream_response.status_code.is_success() {
        state
            .cache
            .put(embedding, upstream_response.response_body.clone())?;
    }

    let mut response = (
        upstream_response.status_code,
        upstream_response.header_map,
        upstream_response.response_body,
    )
        .into_response();

    debug!("Cache miss - calling the upstream LLM provider");
    CACHE_MISS.inc();
    response.extensions_mut().insert(CacheStatus::Miss);

    Ok(response)
}

#[cfg(test)]
mod tests {
    use crate::clients::client::UpstreamResponse;
    use crate::providers::ProviderType;
    use crate::{
        app_state::AppState, cache::cache::MockCache, cache::error::CacheError,
        clients::client::MockClient, embedding::service::MockEmbeddingService,
        endpoints::chat::error::CompletionError, endpoints::chat::handler::completions,
    };
    use axum::extract::State;
    use axum::http::{HeaderMap, StatusCode};
    use axum::response::{IntoResponse, Response};
    use mockall::predicate::eq;
    use serde_json::json;
    use std::sync::Arc;

    #[tokio::test]
    async fn should_return_error_on_cache_failure() {
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

        // set up cache mock and assert we don't reach it
        let mut mock_client = MockClient::new();
        mock_client.expect_post_http_request().times(0);

        // put mocked objects into the appstate
        let app_state = Arc::new(AppState {
            embedding_service: Box::new(mock_embed),
            cache: Box::new(mock_cache),
            http_client: Box::new(mock_client),
        });

        let request_body = json!({
            "messages": [{
                "role": "user",
                "content": prompt
            }],
            "model": "gpt-4"
        });

        let mut headers = HeaderMap::new();
        headers.insert("Authorization", "Bearer dummy".parse().unwrap());
        headers.insert("X-LLM-PROXY-UPSTREAM", "http://localhost".parse().unwrap());

        // when
        let result = completions(
            State(app_state),
            headers,
            axum::Json(request_body),
            ProviderType::OpenAI,
        )
        .await;

        // then
        match result {
            Err(CompletionError::InternalCacheError(_)) => {}
            _ => panic!("Expected CompletionError::Internal"),
        }
    }

    #[tokio::test]
    async fn should_return_error_on_embedding_failure() {
        // given
        let prompt = "bad prompt";

        // embedding fails
        let mut mock_embed = MockEmbeddingService::new();
        mock_embed.expect_embed().times(1).returning(|_| {
            Err(crate::embedding::error::EmbeddingError::GenerationError(
                "Embedding error".into(),
            ))
        });

        // cache should not be touched
        let mut mock_cache = MockCache::new();
        mock_cache.expect_get_if_present().times(0);

        // client should not be called either
        let mut mock_client = MockClient::new();
        mock_client.expect_post_http_request().times(0);

        let app_state = Arc::new(AppState {
            embedding_service: Box::new(mock_embed),
            cache: Box::new(mock_cache),
            http_client: Box::new(mock_client),
        });

        let request_body = json!({
            "messages": [{
                "role": "user",
                "content": prompt
            }],
            "model": "gpt-4"
        });

        let mut headers = HeaderMap::new();
        headers.insert("Authorization", "Bearer dummy".parse().unwrap());
        headers.insert("X-LLM-PROXY-UPSTREAM", "http://localhost".parse().unwrap());

        let result = completions(
            State(app_state),
            headers,
            axum::Json(request_body),
            ProviderType::OpenAI,
        )
        .await;

        match result {
            Err(CompletionError::InternalEmbeddingError(_)) => {}
            _ => panic!("Expected CompletionError::InternalEmbeddingError"),
        }
    }

    #[tokio::test]
    async fn should_return_cached_response_on_cache_hit() {
        // given
        let prompt = "What is semcache?";
        let embedding = vec![0.1, 0.2, 0.3];
        let completion_text = "Semcache is a semantic cache.";
        let completion_json = format!(
            r#"{{"choices":[{{"message":{{"content":"{}"}}}}]}}"#,
            completion_text
        );

        // embed returns vector
        let mut mock_embed = MockEmbeddingService::new();
        mock_embed.expect_embed().times(2).returning({
            let embedding_clone = embedding.clone();
            move |_| Ok(embedding_clone.clone())
        });

        // cache miss
        let mut mock_cache = MockCache::new();
        mock_cache.expect_get_if_present().times(2).returning({
            let completion_clone = completion_json.clone();
            move |_| Ok(Some(completion_clone.clone().into_bytes()))
        });

        // verify put is not called
        mock_cache
            .expect_put()
            .times(0)
            .returning(|_, _| unreachable!());

        // verify client is not called
        let mut mock_client = MockClient::new();
        mock_client
            .expect_post_http_request()
            .times(0)
            .returning(|_, _, _| unreachable!());

        let app_state = Arc::new(AppState {
            embedding_service: Box::new(mock_embed),
            cache: Box::new(mock_cache),
            http_client: Box::new(mock_client),
        });

        // Test OpenAI message
        let request_body = json!({
            "messages": [{
                "role": "user",
                "content": prompt
            }],
            "model": "gpt-4"
        });

        let mut headers = HeaderMap::new();
        headers.insert("Authorization", "Bearer dummy".parse().unwrap());

        let result = completions(
            State(app_state.clone()),
            headers,
            axum::Json(request_body),
            ProviderType::OpenAI,
        )
        .await;

        assert!(result.is_ok());
        let response = result.unwrap().into_response();
        assert_eq!(response.status(), StatusCode::OK);
        let response_json = extract_response(response).await;
        assert_eq!(response_json, completion_json);

        // Test Anthropic endpoint
        let request_body = json!({
            "model": "claude-3-5-sonnet-20241022",
            "max_tokens": 1024,
            "messages": [{
                "role": "user",
                "content": prompt
            }]
        });

        let mut headers = HeaderMap::new();
        headers.insert("Authorization", "Bearer dummy".parse().unwrap());

        // when
        let result = completions(
            State(app_state),
            headers,
            axum::Json(request_body),
            ProviderType::Anthropic,
        )
        .await;

        // then
        assert!(result.is_ok());
        let response = result.unwrap().into_response();
        assert_eq!(response.status(), StatusCode::OK);
        let response_json = extract_response(response).await;
        assert_eq!(response_json, completion_json);
    }

    #[tokio::test]
    async fn should_call_upstream_on_cache_miss_and_caches_response() {
        // given
        let prompt = "What is semcache?";
        let embedding = vec![0.1, 0.2, 0.3];
        let completion_text = "Semcache is a semantic cache.";
        let completion_json = format!(
            r#"{{"choices":[{{"message":{{"content":"{}"}}}}]}}"#,
            completion_text
        );

        // embed returns vector
        let mut mock_embed = MockEmbeddingService::new();
        mock_embed.expect_embed().times(1).returning({
            let embedding_clone = embedding.clone();
            move |_| Ok(embedding_clone.clone())
        });

        // cache miss
        let mut mock_cache = MockCache::new();
        mock_cache
            .expect_get_if_present()
            .times(1)
            .returning(|_| Ok(None));

        // verify put is called once
        mock_cache
            .expect_put()
            .times(1)
            .with(
                eq(embedding.clone()),
                eq(completion_json.clone().into_bytes()),
            )
            .returning(|_, _| Ok(()));

        // upstream response simulation
        let mut mock_client = MockClient::new();
        mock_client.expect_post_http_request().times(1).returning({
            let completion_clone = completion_json.clone();
            move |_, _, _| {
                let resp = UpstreamResponse {
                    status_code: StatusCode::OK,
                    header_map: HeaderMap::new(),
                    response_body: completion_clone.clone().into_bytes(),
                };
                Ok(resp)
            }
        });

        let app_state = Arc::new(AppState {
            embedding_service: Box::new(mock_embed),
            cache: Box::new(mock_cache),
            http_client: Box::new(mock_client),
        });

        let request_body = json!({
            "messages": [{
                "role": "user",
                "content": prompt
            }],
            "model": "gpt-4"
        });

        let mut headers = HeaderMap::new();
        headers.insert("Authorization", "Bearer dummy".parse().unwrap());
        headers.insert("X-LLM-PROXY-UPSTREAM", "http://localhost".parse().unwrap());

        let result = completions(
            State(app_state),
            headers,
            axum::Json(request_body),
            ProviderType::OpenAI,
        )
        .await;

        // then
        assert!(result.is_ok());
        let response = result.unwrap().into_response();
        assert_eq!(response.status(), StatusCode::OK);
        let response_json = extract_response(response).await;
        assert_eq!(response_json, completion_json);
    }

    #[tokio::test]
    async fn should_not_cache_response_when_non_200_ok_response() {
        // given
        let prompt = "What is semcache?";
        let embedding = vec![0.1, 0.2, 0.3];
        let response_body = "Error from LLM!";

        // embed returns vector
        let mut mock_embed = MockEmbeddingService::new();
        mock_embed.expect_embed().times(1).returning({
            let embedding_clone = embedding.clone();
            move |_| Ok(embedding_clone.clone())
        });

        // cache miss
        let mut mock_cache = MockCache::new();
        mock_cache
            .expect_get_if_present()
            .times(1)
            .returning(|_| Ok(None));

        // verify put is called once
        mock_cache.expect_put().times(0);

        // upstream response simulation
        let mut mock_client = MockClient::new();
        mock_client.expect_post_http_request().times(1).returning({
            move |_, _, _| {
                let resp = UpstreamResponse {
                    status_code: StatusCode::UNAUTHORIZED,
                    header_map: HeaderMap::new(),
                    response_body: Vec::from(response_body),
                };
                Ok(resp)
            }
        });

        let app_state = Arc::new(AppState {
            embedding_service: Box::new(mock_embed),
            cache: Box::new(mock_cache),
            http_client: Box::new(mock_client),
        });

        let request_body = json!({
            "messages": [{
                "role": "user",
                "content": prompt
            }],
            "model": "gpt-4"
        });

        let mut headers = HeaderMap::new();
        headers.insert("Authorization", "Bearer dummy".parse().unwrap());
        headers.insert("X-LLM-PROXY-UPSTREAM", "http://localhost".parse().unwrap());

        let result = completions(
            State(app_state),
            headers,
            axum::Json(request_body),
            ProviderType::OpenAI,
        )
        .await;

        // then
        assert!(result.is_ok());
        let response = result.unwrap().into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let response = extract_response(response).await;
        assert_eq!(response, response_body.to_string());
    }

    async fn extract_response(response: Response) -> String {
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        String::from_utf8(body.to_vec()).unwrap()
    }
}

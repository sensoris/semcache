use axum::http::HeaderName;
use axum::{
    extract::{Json, State},
    http::{StatusCode, header::HeaderMap},
    response::IntoResponse,
};
use serde_json::Value;
use std::sync::{Arc, LazyLock};
use tracing::{debug, info};

use super::error::CompletionError;
use crate::app_state::AppState;
use crate::providers::Provider;
use crate::utils::json_extract::extract_prompt_from_path;

// HEADERS
static PROXY_UPSTREAM_HEADER: HeaderName = HeaderName::from_static("x-llm-proxy-upstream");
static PROXY_PROMPT_LOCATION_HEADER: HeaderName = HeaderName::from_static("x-llm-prompt");
static HOP_HEADERS: LazyLock<[HeaderName; 10]> = LazyLock::new(|| {
    [
        HeaderName::from_static("connection"),
        HeaderName::from_static("te"),
        HeaderName::from_static("trailer"),
        HeaderName::from_static("keep-alive"),
        HeaderName::from_static("proxy-connection"),
        HeaderName::from_static("proxy-authenticate"),
        HeaderName::from_static("proxy-authorization"),
        HeaderName::from_static("transfer-encoding"),
        HeaderName::from_static("upgrade"),
        // todo - why do we need to remove this?
        HeaderName::from_static("content-length"),
    ]
});

pub async fn completions<P: Provider>(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    request_body: Json<Value>,
    provider: P,
) -> Result<impl IntoResponse, CompletionError> {
    let prompt = extract_prompt_from_path(&request_body, provider.prompt_path())?;
    info!("prompt: {prompt}");
    let embedding = state.embedding_service.embed(&prompt)?;

    if let Some(saved_response) = state.cache.get_if_present(&embedding)? {
        info!("CACHE HIT");
        // Return cached response with 200 OK and minimal headers
        let mut response_headers = HeaderMap::new();
        response_headers.insert("X-Cache-Status", "hit".parse().unwrap());
        return Ok((StatusCode::OK, response_headers, saved_response));
    };

    info!("CACHE_MISS");

    // otherwise, send upstream request
    let upstream_url = provider.upstream_url();
    let upstream_headers = prepare_upstream_headers(headers, provider);

    let upstream_response = state
        .http_client
        .post_http_request(upstream_headers, upstream_url.clone(), &request_body)
        .await?;

    let mut response_headers = upstream_response.headers().clone();
    let status = upstream_response.status();

    // Get the body as bytes (not JSON)
    let body_bytes = upstream_response
        .bytes()
        .await
        .map_err(|e| CompletionError::Upstream(e))?;

    // todo, am converting into string, maybe its okay to return bytes?
    let response_string = String::from_utf8_lossy(&body_bytes).to_string();

    state.cache.put(embedding, response_string.clone())?;

    // todo prepare response headers
    remove_hop_headers(&mut response_headers);

    Ok((status, response_headers, response_string))
}

fn prepare_upstream_headers<P: Provider>(headers: HeaderMap, provider: P) -> HeaderMap {
    let mut upstream_headers = headers.clone();

    remove_hop_headers(&mut upstream_headers);

    // remove semcache headers
    upstream_headers.remove(&PROXY_UPSTREAM_HEADER);
    upstream_headers.remove(&PROXY_PROMPT_LOCATION_HEADER);

    // add host for request to be accepted
    upstream_headers.insert("host", provider.header_host());
    upstream_headers
}

fn remove_hop_headers(headers: &mut HeaderMap) {
    debug!("Removing hop headers");

    for header in &*HOP_HEADERS {
        headers.remove(header);
    }
}

#[cfg(test)]
mod tests {
    use crate::providers::anthropic::Anthropic;
    use crate::providers::openai::OpenAI;
    use crate::{
        app_state::AppState, cache::cache::MockCache, cache::error::CacheError,
        clients::client::MockClient, embedding::service::MockEmbeddingService,
        endpoints::chat::error::CompletionError, endpoints::chat::handler::completions,
    };
    use axum::extract::State;
    use axum::http::{self, HeaderMap, StatusCode};
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
        let mut mock_cache = MockCache::new();
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
        let result = completions(State(app_state), headers, axum::Json(request_body), OpenAI).await;

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

        let result = completions(State(app_state), headers, axum::Json(request_body), OpenAI).await;

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
            move |_| Ok(Some(completion_clone.clone()))
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
            OpenAI,
        )
        .await;

        assert!(result.is_ok());
        let response = result.unwrap().into_response();
        assert_eq!(response.status(), StatusCode::OK);
        let response_json = extract_json_response(response).await;
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
            Anthropic,
        )
        .await;

        // then
        assert!(result.is_ok());
        let response = result.unwrap().into_response();
        assert_eq!(response.status(), StatusCode::OK);
        let response_json = extract_json_response(response).await;
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
            .with(eq(embedding.clone()), eq(completion_json.clone()))
            .returning(|_, _| Ok(()));

        // upstream response simulation
        let mut mock_client = MockClient::new();
        mock_client.expect_post_http_request().times(1).returning({
            let completion_clone = completion_json.clone();
            move |_, _, _| {
                let resp = reqwest::Response::from(http::Response::new(completion_clone.clone()));
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

        let result = completions(State(app_state), headers, axum::Json(request_body), OpenAI).await;

        // then
        assert!(result.is_ok());
        let response = result.unwrap().into_response();
        assert_eq!(response.status(), StatusCode::OK);
        let response_json = extract_json_response(response).await;
        assert_eq!(response_json, completion_json);
    }

    async fn extract_json_response(response: Response) -> String {
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        String::from_utf8(body.to_vec()).unwrap()
    }
}

use axum::{
    extract::{Json, State},
    http::header::HeaderMap,
};
use std::sync::Arc;
use tracing::info;

use crate::app_state::AppState;
use crate::metrics::CHAT_COMPLETIONS;
use url::Url;

use super::{
    dto::{CompletionRequest, CompletionResponse},
    error::CompletionError,
};

// CONSTANTS
const PROXY_UPSTREAM_HEADER: &'static str = "X-LLM-PROXY-UPSTREAM";

pub async fn completions(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request_body): Json<CompletionRequest>,
) -> Result<CompletionResponse, CompletionError> {
    CHAT_COMPLETIONS.inc();

    let prompt = &request_body
        .messages
        .get(0)
        .ok_or(CompletionError::InvalidRequest(
            "No messages in request".into(),
        ))?
        .content;

    let embedding = state.embedding_service.embed(&prompt)?;

    // return early if cache hit
    if let Some(saved_response) = state.cache.get_if_present(&embedding)? {
        info!("CACHE HIT");
        return Ok(CompletionResponse::from_cache(saved_response)?);
    };

    info!("CACHE_MISS");

    // otherwise, send upstream request
    let auth_token = extract_auth_token(&headers)?;
    let upstream_url = extract_proxy_upstream(&headers)?;
    let reqwest_response = state
        .http_client
        .send_completion_request(auth_token, upstream_url, &request_body)
        .await?;
    let response = CompletionResponse::from_reqwest(reqwest_response).await?;

    // save returned response in cache
    let response_string: String = (&response).try_into()?;
    state.cache.put(embedding, response_string)?;

    Ok(response)
}

fn extract_auth_token(headers: &HeaderMap) -> Result<&str, CompletionError> {
    // extract auth token
    headers
        .get(axum::http::header::AUTHORIZATION)
        .ok_or(CompletionError::InvalidRequest(String::from(
            "Missing authorization header",
        )))?
        .to_str()
        .map_err(|error| {
            CompletionError::InvalidRequest(format!(
                "authorization header could not be parsed as a string, {}",
                error
            ))
        })
}

fn extract_proxy_upstream(headers: &HeaderMap) -> Result<Url, CompletionError> {
    let raw = headers.get(PROXY_UPSTREAM_HEADER).ok_or_else(|| {
        CompletionError::InvalidRequest(format!("Missing {} header", PROXY_UPSTREAM_HEADER))
    })?;

    let url_str = raw.to_str().map_err(|e| {
        CompletionError::InvalidRequest(format!(
            "{} header is not valid UTF-8: {}",
            PROXY_UPSTREAM_HEADER, e
        ))
    })?;

    Url::parse(url_str).map_err(|e| {
        CompletionError::InvalidRequest(format!(
            "{} header is not a valid URL: {}, error: {}",
            PROXY_UPSTREAM_HEADER, url_str, e
        ))
    })
}

#[cfg(test)]
mod tests {

    use axum::extract::State;
    use axum::http::HeaderMap;
    use std::sync::Arc;

    use crate::{
        app_state::AppState,
        cache::cache::MockCache,
        cache::error::CacheError,
        clients::client::MockClient,
        embedding::service::MockEmbeddingService,
        endpoints::chat::dto::{CompletionRequest, Message},
        endpoints::chat::error::CompletionError,
        endpoints::chat::handler::completions,
    };

    #[tokio::test]
    async fn test_completions_returns_error_on_cache_failure() {
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
        mock_client.expect_send_completion_request().times(0);

        // put mocked objects into the appstate
        let app_state = Arc::new(AppState {
            embedding_service: Box::new(mock_embed),
            cache: Box::new(mock_cache),
            http_client: Box::new(mock_client),
        });

        let request_body = CompletionRequest {
            messages: vec![Message {
                role: "user".into(),
                content: prompt.into(),
            }],
            model: "gpt-4o".into(),
        };

        let mut headers = HeaderMap::new();
        headers.insert("Authorization", "Bearer dummy".parse().unwrap());
        headers.insert("X-LLM-PROXY-UPSTREAM", "http://localhost".parse().unwrap());

        // when
        let result = completions(State(app_state), headers, axum::Json(request_body)).await;

        // then
        match result {
            Err(CompletionError::InternalCacheError(_)) => {}
            _ => panic!("Expected CompletionError::Internal"),
        }
    }

    #[tokio::test]
    async fn test_completions_returns_error_on_embedding_failure() {
        // given
        let prompt = "bad prompt";

        // embedding fails
        let mut mock_embed = MockEmbeddingService::new();
        mock_embed.expect_embed().times(1).returning(|_| {
            Err(crate::embedding::error::EmbeddingError::GenerationError(
                "bumba".into(),
            ))
        });

        // cache should not be touched
        let mut mock_cache = MockCache::new();
        mock_cache.expect_get_if_present().times(0);

        // client should not be called either
        let mut mock_client = MockClient::new();
        mock_client.expect_send_completion_request().times(0);

        let app_state = Arc::new(AppState {
            embedding_service: Box::new(mock_embed),
            cache: Box::new(mock_cache),
            http_client: Box::new(mock_client),
        });

        let request_body = CompletionRequest {
            messages: vec![Message {
                role: "user".into(),
                content: prompt.into(),
            }],
            model: "gpt-4o".into(),
        };

        let mut headers = HeaderMap::new();
        headers.insert("Authorization", "Bearer dummy".parse().unwrap());
        headers.insert("X-LLM-PROXY-UPSTREAM", "http://localhost".parse().unwrap());

        // when
        let result = super::completions(State(app_state), headers, axum::Json(request_body)).await;

        // then
        match result {
            Err(CompletionError::InternalEmbeddingError(_)) => {}
            _ => panic!("Expected CompletionError::InternalEmbeddingError"),
        }
    }
}

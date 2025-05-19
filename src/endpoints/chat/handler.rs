use axum::{
    extract::{Json, State},
    http::header::HeaderMap,
};
use std::sync::Arc;

use crate::app_state::AppState;
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
    let prompt = &request_body
        .messages
        .get(0)
        .ok_or(CompletionError::InvalidRequest(
            "No messages in request".into(),
        ))?
        .content;

    // return early if cache hit
    if let Some(saved_response) = state.cache.get_if_present(&prompt)? {
        println!("CACHE HIT");
        return Ok(CompletionResponse::from_cache(saved_response)?);
    };

    println!("CACHE_MISS");

    // otherwise, send upstream request
    let auth_token = extract_auth_token(&headers)?;
    let upstream_url = extract_proxy_upstream(&headers)?;
    let reqwest_response =
        send_request(state.clone(), auth_token, upstream_url, &request_body).await?;
    let response = CompletionResponse::from_reqwest(reqwest_response).await?;

    // save returned response in cache
    let response_string: String = (&response).try_into()?;
    state.cache.put(prompt, response_string)?;

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

async fn send_request(
    state: Arc<AppState>,
    auth_token: &str,
    upstream_url: Url,
    request_body: &CompletionRequest,
) -> Result<reqwest::Response, reqwest::Error> {
    let response = state
        .http_client
        .post(upstream_url)
        .header(reqwest::header::AUTHORIZATION, auth_token)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .json(&request_body)
        .send()
        .await?;
    Ok(response)
}

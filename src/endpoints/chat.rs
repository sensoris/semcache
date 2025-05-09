use core::panic;

use axum::{extract::Json, http::header::HeaderMap};

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct CompletionRequest {
    pub model: String,
    pub messages: Vec<Message>,
}

#[derive(Deserialize, Debug)]
pub struct Message {
    pub role: String,
    pub content: String,
}

pub async fn chat_completions(headers: HeaderMap, Json(request_body): Json<CompletionRequest>) {
    println!(
        "model: {}, prompt: {}, role: {}",
        request_body.model, request_body.messages[0].content, request_body.messages[0].role
    );
    
    let auth_token = match headers.get("Authorization") {
        Some(token) => token,
        None => panic!("couldnt find auth token")
    };
    println!("Auth token: {}", auth_token.to_str().expect("auth_token was not a valid string"));
}

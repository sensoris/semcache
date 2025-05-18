mod app_state;
mod embedding;
mod endpoints;
mod cache;

use app_state::AppState;
use axum::{Router, routing::get, routing::post};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let shared_state = Arc::new(AppState::new());

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route(
            "/chat/completions",
            post(endpoints::chat::handler::completions),
        )
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

mod app_state;
mod endpoints;
use app_state::AppState;
use axum::{Router, routing::get, routing::post};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // initialize shared state
    let shared_state = Arc::new(AppState::new());

    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/chat/completions", post(endpoints::chat::chat_completions))
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

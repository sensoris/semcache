mod app_state;
mod cache;
mod embedding;
mod endpoints;
mod utils;
mod metrics;
use app_state::AppState;
use axum::{Router, routing::get, routing::post};
use std::sync::Arc;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    metrics::initialize_metrics_collection();

    let shared_state = Arc::new(AppState::new());

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route(
            "/chat/completions",
            post(endpoints::chat::handler::completions),
        )
        .route("/admin", get(endpoints::admin::handler::dashboard))
        .route(
            "/api/metrics",
            get(endpoints::metrics::handler::get_metrics),
        )
        .nest_service("/static", ServeDir::new("assets"))
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

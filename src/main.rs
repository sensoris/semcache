mod app_state;
mod cache;
mod embedding;
mod endpoints;
mod metrics;
mod utils;
use app_state::AppState;
use axum::{Router, routing::get, routing::post};
use std::sync::Arc;
use tower_http::services::ServeDir;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    // TODO: this should come from config
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("info"))
        .init();

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

    let port = 8080;

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .unwrap();
    info!(
        "semcache-rs application started successfully on port {}",
        port
    );
    axum::serve(listener, app).await.unwrap();
}

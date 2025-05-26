mod app_state;
mod cache;
mod embedding;
mod endpoints;
mod metrics;
mod utils;
use app_state::AppState;
use axum::{Router, routing::get, routing::post};
use std::sync::Arc;
use tokio::signal;
use tower_http::services::ServeDir;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    // TODO (after config file ticket): this should come from config
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
        .unwrap_or_else(|err| {
            error!(error = ?err);
            panic!("Failed to start listener")
        });
    info!(
        "semcache-rs application started successfully on port {}",
        port
    );

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap_or_else(|err| {
            error!(error = ?err);
            panic!("Failed to start axum server")
        });
}

// Inspired by https://github.com/tokio-rs/axum/blob/main/examples/graceful-shutdown/src/main.rs
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C, shutting down gracefully");
        },
        _ = terminate => {
            info!("Received SIGTERM, shutting down gracefully");
        },
    }
}

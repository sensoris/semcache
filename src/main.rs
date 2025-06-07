mod app_state;
mod cache;
mod clients;
mod config;
mod embedding;
mod endpoints;
mod metrics;
mod providers;
mod utils;

use crate::config::get_eviction_policy;
use crate::endpoints::chat::provider_handlers::{
    anthropic_handler, generic_handler, openai_handler,
};
use crate::endpoints::metrics::handler::prometheus_metrics_handler;
use crate::metrics::metrics::{init_metrics, track_metrics};
use crate::providers::OPEN_AI_REST_PATH;
use app_state::AppState;
use axum::http::StatusCode;
use axum::{Router, routing::get, routing::post};
use config::{get_log_level, get_port, get_similarity_threshold};
use providers::ProviderType;
use std::sync::Arc;
use tokio::signal;
use tower_http::services::ServeDir;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

const CONFIG_FILE: &str = "config.yaml";
const STARTUP_MESSAGE: &str = "Semcache started successfully";

#[tokio::main]
async fn main() {
    let config = config::from_file(CONFIG_FILE);

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(
            get_log_level(&config).unwrap_or("debug".into()),
        ))
        .init();

    info!("Starting application...");

    // metrics setup
    init_metrics();

    let similarity_threshold = get_similarity_threshold(&config).unwrap_or(0.90) as f32;
    let eviction_policy = get_eviction_policy(&config).unwrap_or_else(|err| {
        error!(?err, "Missing or malformed eviction policy from conf");
        panic!("Missing or malformed eviction policy in config")
    });

    let shared_state = Arc::new(AppState::new(similarity_threshold, eviction_policy));

    let provider_routes = Router::new()
        // Provider endpoints
        .route(ProviderType::OpenAI.path(), post(openai_handler))
        .route(OPEN_AI_REST_PATH, post(openai_handler))
        .route(ProviderType::Anthropic.path(), post(anthropic_handler))
        .route(ProviderType::Generic.path(), post(generic_handler))
        .layer(axum::middleware::from_fn(track_metrics)); // Apply middleware only to these routes

    let app = Router::new()
        // healthcheck
        .route("/", get(|| async { StatusCode::OK }))
        // Provider endpoints
        .merge(provider_routes)
        // Prometheus metrics
        .route("/metrics", get(prometheus_metrics_handler))
        // Admin dashboard
        .route("/admin", get(endpoints::admin::handler::dashboard))
        // Dashboard metrics
        .route(
            "/dashboard-metrics",
            get(endpoints::metrics::handler::dashboard_metrics_handler),
        )
        .nest_service("/static", ServeDir::new("assets"))
        .with_state(shared_state);

    let port = get_port(&config).unwrap_or(8080);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .unwrap_or_else(|err| {
            error!(error = ?err);
            panic!("Failed to start listener")
        });

    info!("{}", STARTUP_MESSAGE);
    info!("Ready to receive requests on {port}");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap_or_else(|err| {
            error!(error = ?err);
            panic!("Failed to start axum server")
        });
}

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

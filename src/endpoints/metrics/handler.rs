use crate::metrics::dashboard::{DashboardMetricsResponse, dashboard_metrics};
use axum::Json;
use prometheus::{TextEncoder, default_registry};

pub async fn dashboard_metrics_handler() -> Json<DashboardMetricsResponse> {
    let metrics_response = dashboard_metrics();
    Json(metrics_response)
}

pub async fn prometheus_metrics_handler() -> String {
    let encoder = TextEncoder::new();
    let metric_families = default_registry().gather();
    encoder.encode_to_string(&metric_families).unwrap()
}

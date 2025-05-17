use crate::metrics::{MetricsResponse, metrics};
use axum::Json;

pub async fn get_metrics() -> Json<MetricsResponse> {
    let metrics_response = metrics();
    Json(metrics_response)
}

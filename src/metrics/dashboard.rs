use crate::metrics::metrics::MEM_USAGE_KB;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

const METRICS_HISTORY_PATH: &str = "assets/metrics_history.json";
const MAX_LENGTH: usize = 50;

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum ChartType {
    Bar,
    Line,
    Doughnut,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DashboardMetrics {
    pub name: String,
    pub value: i64,
    pub chart_type: ChartType,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DashboardMetricsResponse {
    pub timestamp: DateTime<Utc>,
    pub metrics: Vec<DashboardMetrics>,
}

pub fn dashboard_metrics() -> DashboardMetricsResponse {
    let metrics = vec![
        DashboardMetrics {
            name: "Chat completion requests".to_string(),
            // todo implement with real metrics
            value: 1,
            chart_type: ChartType::Line,
        },
        DashboardMetrics {
            name: "Memory usage (mb) - only available in Linux systems".to_string(),
            value: MEM_USAGE_KB.get() / 1024,
            chart_type: ChartType::Line,
        },
    ];

    DashboardMetricsResponse {
        timestamp: Utc::now(),
        metrics,
    }
}

pub fn update_dashboard_history() {
    if !Path::new(METRICS_HISTORY_PATH).exists() {
        fs::write(METRICS_HISTORY_PATH, "[]").expect("Failed to create metrics history file");
    }

    let current_metrics = dashboard_metrics();
    let mut history = dashboard_metrics_history();

    history.push(current_metrics);
    history = limit_history_length(history);

    // Write updated history back to the file
    if let Ok(json_string) = serde_json::to_string_pretty(&history) {
        let _ = fs::write(METRICS_HISTORY_PATH, json_string);
    }
}

fn dashboard_metrics_history() -> Vec<DashboardMetricsResponse> {
    let history_content =
        fs::read_to_string(METRICS_HISTORY_PATH).unwrap_or_else(|_| "[]".to_string());

    let history: Vec<DashboardMetricsResponse> =
        serde_json::from_str(&history_content).unwrap_or_else(|_| Vec::new());

    history
}

fn limit_history_length(history: Vec<DashboardMetricsResponse>) -> Vec<DashboardMetricsResponse> {
    if history.len() > MAX_LENGTH {
        history
            .clone()
            .into_iter()
            .skip(history.len() - MAX_LENGTH)
            .collect()
    } else {
        history
    }
}

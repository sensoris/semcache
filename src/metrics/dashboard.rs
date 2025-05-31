use crate::metrics::metrics::{CACHE_HIT, CACHE_MISS, CACHE_SIZE, MEM_USAGE_KB};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

const METRICS_HISTORY_PATH: &str = "assets/metrics_history.json";
const MAX_LENGTH: usize = 50;

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum ChartType {
    Bar,
    Line,
    Doughnut,
    StatCard,
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
            name: "Num entries in the cache".to_string(),
            value: CACHE_SIZE.get(),
            chart_type: ChartType::StatCard,
        },
        DashboardMetrics {
            name: "Total chat completion requests".to_string(),
            value: (CACHE_HIT.get() + CACHE_MISS.get()) as i64,
            chart_type: ChartType::StatCard,
        },
        DashboardMetrics {
            name: "Cache hits".to_string(),
            value: CACHE_HIT.get() as i64,
            chart_type: ChartType::Line,
        },
        DashboardMetrics {
            name: "Cache miss".to_string(),
            value: CACHE_MISS.get() as i64,
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

pub async fn update_dashboard_history() {
    tokio::fs::write(METRICS_HISTORY_PATH, "[]")
        .await
        .expect("Failed to create metrics history file");

    let current_metrics = dashboard_metrics();
    let mut history = dashboard_metrics_history().await;
    history.push(current_metrics);
    history = limit_history_length(history);

    // Write updated history back to the file
    if let Ok(json_string) = serde_json::to_string_pretty(&history) {
        let _ = tokio::fs::write(METRICS_HISTORY_PATH, json_string).await;
    }
}

async fn dashboard_metrics_history() -> Vec<DashboardMetricsResponse> {
    let history_content = tokio::fs::read_to_string(METRICS_HISTORY_PATH)
        .await
        .unwrap_or_else(|_| "[]".to_string());

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

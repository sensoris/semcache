use chrono::{DateTime, Utc};
use prometheus::{IntCounter, IntGauge, register_int_counter, register_int_gauge};
use serde::{Deserialize, Serialize};
use serde_json::{self};
use std::fs;
use std::path::Path;
use std::sync::LazyLock;
use tokio::task;
use tokio::time::{self, Duration};
use tracing::{debug, error};

use crate::utils::cgroup_utils;

const METRICS_HISTORY_PATH: &str = "assets/metrics_history.json";

pub static CHAT_COMPLETIONS: LazyLock<IntCounter> = LazyLock::new(|| {
    register_int_counter!("incoming_requests", "Incoming Requests").unwrap_or_else(|err| {
        error!(error = ?err);
        panic!("Issue creating chat completions usage metric")
    })
});

pub static MEM_USAGE_KB: LazyLock<IntGauge> = LazyLock::new(|| {
    register_int_gauge!("memory_usage", "Application memory usage in kilobytes (only available in Linux systems)").unwrap_or_else(
        |err| {
            error!(error = ?err);
            panic!("Issue creating memory usage metric")
        },
    )
});

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum ChartType {
    Bar,
    Line,
    Doughnut,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Metrics {
    pub name: String,
    pub value: i64,
    pub chart_type: ChartType,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MetricsResponse {
    pub timestamp: DateTime<Utc>,
    pub metrics: Vec<Metrics>,
}

pub fn metrics() -> MetricsResponse {
    let metrics = vec![
        Metrics {
            name: "Chat completion requests".to_string(),
            value: CHAT_COMPLETIONS.get() as i64,
            chart_type: ChartType::Line,
        },
        Metrics {
            name: "Memory usage (mb) - only available in Linux systems".to_string(),
            value: MEM_USAGE_KB.get() / 1024,
            chart_type: ChartType::Line,
        },
    ];

    MetricsResponse {
        timestamp: Utc::now(),
        metrics: metrics,
    }
}

pub fn initialize_metrics_collection() {
    if !Path::new(METRICS_HISTORY_PATH).exists() {
        fs::write(METRICS_HISTORY_PATH, "[]").expect("Failed to create metrics history file");
    }

    // Start the background task for metrics collection
    start_metrics_collection();
}

const MAX_LENGTH: usize = 50;

fn start_metrics_collection() {
    task::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(30));

        loop {
            interval.tick().await;

            update_mem_usage_metric();

            let current_metrics = metrics();
            let mut history = history();

            history.push(current_metrics);
            history = limit_history_length(history);

            // Write updated history back to the file
            if let Ok(json_string) = serde_json::to_string_pretty(&history) {
                let _ = fs::write(METRICS_HISTORY_PATH, json_string);
            }
        }
    });
}

fn update_mem_usage_metric() {
    match cgroup_utils::read_cgroup_v2_memory_kb() {
        Some(used_memory_kb) => MEM_USAGE_KB.set((used_memory_kb) as i64),
        None => debug!("could not report current memory usage"),
    }
}

fn history() -> Vec<MetricsResponse> {
    let history_content =
        fs::read_to_string(METRICS_HISTORY_PATH).unwrap_or_else(|_| "[]".to_string());

    let history: Vec<MetricsResponse> =
        serde_json::from_str(&history_content).unwrap_or_else(|_| Vec::new());

    history
}

fn limit_history_length(history: Vec<MetricsResponse>) -> Vec<MetricsResponse> {
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

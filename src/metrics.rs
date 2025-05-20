use chrono::{DateTime, Utc};
use prometheus::{IntCounter, register_int_counter};
use serde::{Deserialize, Serialize};
use serde_json::{self};
use std::fs;
use std::path::Path;
use std::sync::LazyLock;
use tokio::task;
use tokio::time::{self, Duration};

const METRICS_HISTORY_PATH: &str = "assets/metrics_history.json";

// Note:
// Saw online that we shouldn't be using lazy_static and instead use the new built in LazyLock
// But I found it very complicated to understand how to use it and the prometheus examples
// used lazy_static and works smoothly
// https://github.com/tikv/rust-prometheus/blob/master/examples/example_int_metrics.rs
// https://www.reddit.com/r/rust/comments/1iisfzg/lazycell_vs_lazylock_vs_oncecell_vs_oncelock_vs/
pub static CHAT_COMPLETIONS: LazyLock<IntCounter> = LazyLock::new(|| {
    register_int_counter!("incoming_requests", "Incoming Requests")
        .expect("metric can be created")
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
    pub value: u64,
    pub chart_type: ChartType,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MetricsResponse {
    pub timestamp: DateTime<Utc>,
    pub metrics: Vec<Metrics>,
}

pub fn metrics() -> MetricsResponse {
    let metrics = vec![Metrics {
        name: "Chat completion requests".to_string(),
        value: CHAT_COMPLETIONS.get(),
        chart_type: ChartType::Line,
    }];

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

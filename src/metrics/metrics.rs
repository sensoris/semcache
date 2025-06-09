use crate::metrics::dashboard::update_dashboard_history;
use crate::utils::cgroup_utils;
use axum::extract::{MatchedPath, Request};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::IntoResponse;
use prometheus::{
    HistogramVec, IntCounter, IntGauge, register_histogram_vec, register_int_counter,
    register_int_gauge,
};
use std::sync::LazyLock;
use std::time::Instant;
use tokio::task;
use tokio::time::{self, Duration};
use tracing::{debug, error};

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum CacheStatus {
    Hit,
    Miss,
    NotApplicable, // For non-cacheable requests
}

// Prefix the metric with "semcache_"
macro_rules! metric_name {
    ($name:expr) => {
        concat!("semcache_", $name)
    };
}

// HTTP request duration histogram with labels
pub static CHAT_COMPLETION_HTTP_REQUESTS: LazyLock<HistogramVec> = LazyLock::new(|| {
    register_histogram_vec!(
        metric_name!("chat_completion_http_requests"),
        "Chat completion http requests duration in seconds",
        &["method", "path", "status", "cache_status"]
    )
    .unwrap_or_else(|err| {
        error!(error = ?err);
        panic!("Issue creating chat_completion_http_requests metric")
    })
});

pub static MEM_USAGE_KB: LazyLock<IntGauge> = LazyLock::new(|| {
    register_int_gauge!(
        metric_name!("memory_usage"),
        "Application memory usage in kilobytes (only available in Linux systems)"
    )
    .unwrap_or_else(|err| {
        error!(error = ?err);
        panic!("Issue creating memory usage metric")
    })
});

pub static CACHE_HIT: LazyLock<IntCounter> = LazyLock::new(|| {
    register_int_counter!(metric_name!("cache_hit"), "Cache hit").unwrap_or_else(|err| {
        error!(error = ?err);
        panic!("Issue creating cache hit metric")
    })
});

pub static CACHE_MISS: LazyLock<IntCounter> = LazyLock::new(|| {
    register_int_counter!(metric_name!("cache_miss"), "Cache miss").unwrap_or_else(|err| {
        error!(error = ?err);
        panic!("Issue creating cache miss metric")
    })
});

pub static CACHE_SIZE: LazyLock<IntGauge> = LazyLock::new(|| {
    register_int_gauge!(
        metric_name!("cache_size"),
        "The number of entries in the cache"
    )
    .unwrap_or_else(|err| {
        error!(error = ?err);
        panic!("Issue creating cache size metric")
    })
});

pub fn init_metrics() {
    initialize_metrics_collection();
}

// Axum middleware to track chat completion requests
pub async fn track_metrics(req: Request, next: Next) -> impl IntoResponse {
    let start = Instant::now();
    let path = if let Some(matched_path) = req.extensions().get::<MatchedPath>() {
        matched_path.as_str().to_owned()
    } else {
        req.uri().path().to_owned()
    };
    let method = req.method().clone();

    let response = next.run(req).await;

    let latency = start.elapsed().as_secs_f64();
    let status = response.status().as_u16().to_string();

    // Extract cache status from response
    let cache_status = response
        .extensions()
        .get::<CacheStatus>()
        .map(|s| match s {
            CacheStatus::Hit => "hit",
            CacheStatus::Miss => "miss",
            CacheStatus::NotApplicable => "n/a",
        })
        .unwrap_or("unknown");

    if response.status() != StatusCode::NOT_FOUND {
        CHAT_COMPLETION_HTTP_REQUESTS
            .with_label_values(&[method.as_str(), &path, &status, cache_status])
            .observe(latency);
    }

    response
}

pub(crate) fn initialize_metrics_collection() {
    // Start the background task for metrics collection
    task::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(30));

        loop {
            interval.tick().await;

            update_mem_usage_metric();

            update_dashboard_history().await;
        }
    });
}

fn update_mem_usage_metric() {
    match cgroup_utils::read_cgroup_v2_memory_kb() {
        Some(used_memory_kb) => MEM_USAGE_KB.set((used_memory_kb) as i64),
        None => debug!("could not report current memory usage"),
    }
}

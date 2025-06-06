use config::{Config, ConfigError};
use serde::Deserialize;
use tracing::{error, warn};

use crate::cache::cache_impl::EvictionPolicy;

const LOG_LEVEL_KEY: &'static str = "log_level";
const PORT_KEY: &'static str = "port";
const SIMILARITY_THRESHOLD_KEY: &'static str = "similarity_threshold";
const EVICTION_POLICY_KEY: &'static str = "eviction_policy";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct EvictionPolicyConfig {
    policy_type: String,
    value: usize,
}

pub fn from_file(config_file_name: &str) -> Config {
    Config::builder()
        .add_source(config::File::with_name(&config_file_name))
        .add_source(config::Environment::with_prefix("SEMCACHE").convert_case(config::Case::Snake))
        .build()
        .unwrap_or_else(|err| {
            error!(error = ?err, "Failed to parse config from file {config_file_name}");
            panic!("Failed to parse config from file {config_file_name}")
        })
}

pub fn get_log_level(conf: &Config) -> Result<String, ConfigError> {
    with_log(|| conf.get_string(LOG_LEVEL_KEY), LOG_LEVEL_KEY)
}

pub fn get_port(conf: &Config) -> Result<i64, ConfigError> {
    with_log(|| conf.get_int(PORT_KEY), PORT_KEY)
}

pub fn get_similarity_threshold(conf: &Config) -> Result<f64, ConfigError> {
    with_log(
        || conf.get_float(SIMILARITY_THRESHOLD_KEY),
        SIMILARITY_THRESHOLD_KEY,
    )
}

pub fn get_eviction_policy(conf: &Config) -> Result<EvictionPolicy, ConfigError> {
    let policy: EvictionPolicyConfig = with_log(
        || conf.get::<EvictionPolicyConfig>(EVICTION_POLICY_KEY),
        EVICTION_POLICY_KEY,
    )?;

    match policy.policy_type.as_str() {
        "entry_limit" => Ok(EvictionPolicy::EntryLimit(policy.value)),
        "memory_limit_bytes" => Ok(EvictionPolicy::MemoryLimitMb(policy.value)),
        other => {
            warn!(ty = other, "Unknown eviction policy type");
            Err(ConfigError::Message(format!(
                "Unknown eviction policy type: {}",
                other
            )))
        }
    }
}

fn with_log<T, F>(get_func: F, conf_field: &'static str) -> Result<T, ConfigError>
where
    F: FnOnce() -> Result<T, ConfigError>,
{
    get_func()
        .inspect_err(|err| warn!(error = ?err, field = conf_field, "Failed to load config value"))
}

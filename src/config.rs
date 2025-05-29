use config::{Config, ConfigError};
use tracing::{error, warn};

const LOG_LEVEL_KEY: &'static str = "log_level";
const PORT_KEY: &'static str = "port";
const SIMILARITY_THRESHOLD_KEY: &'static str = "similarity_threshold";

pub fn from_file(config_file_name: &str) -> Config {
    Config::builder()
        .add_source(config::File::with_name(&config_file_name))
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

fn with_log<T, F>(get_func: F, conf_field: &'static str) -> Result<T, ConfigError>
where
    F: FnOnce() -> Result<T, ConfigError>,
{
    get_func()
        .inspect_err(|err| warn!(error = ?err, field = conf_field, "Failed to load config value"))
}

use std::sync::LazyLock;

use axum::http::HeaderValue;
use url::Url;

// DEFAULTS

// URL's
static ANTHROPIC_DEFAULT_URL: LazyLock<Url> =
    LazyLock::new(|| Url::parse("https://api.anthropic.com/v1/messages").unwrap());
static OPEN_AI_DEFAULT_URL: LazyLock<Url> =
    LazyLock::new(|| Url::parse("https://api.openai.com/v1/chat/completions").unwrap());

// HOST
static ANTHROPIC_DEFAULT_HOST: LazyLock<HeaderValue> =
    LazyLock::new(|| HeaderValue::from_static("api.anthropic.com"));
static OPEN_AI_DEFAULT_HOST: LazyLock<HeaderValue> =
    LazyLock::new(|| HeaderValue::from_static("api.openai.com"));

// REST METHOD PATH
static ANTHROPIC_REST_PATH: &str = "v1/messages";
static OPEN_AI_REST_PATH: &str = "v1/chat/completions";

// JSON PROMPT PATH
static ANTHROPIC_PROMPT_PATH: &str = "$.messages[-1].content";
static OPEN_AI_PROMPT_PATH: &str = "$.messages[-1].content";

pub enum ProviderType {
    Anthropic,
    OpenAI,
    Generic,
}

impl ProviderType {
    pub fn path(&self) -> &'static str {
        match self {
            ProviderType::Anthropic { .. } => ANTHROPIC_REST_PATH,
            ProviderType::OpenAI { .. } => OPEN_AI_REST_PATH,
            ProviderType::Generic => {
                unimplemented!("No default path exists for the generic provider")
            }
        }
    }
    pub fn prompt_json_path(&self) -> &'static str {
        match self {
            ProviderType::Anthropic { .. } => ANTHROPIC_PROMPT_PATH,
            ProviderType::OpenAI { .. } => OPEN_AI_PROMPT_PATH,
            ProviderType::Generic => todo!(),
        }
    }

    pub fn host_header(&self) -> &'static HeaderValue {
        match self {
            ProviderType::Anthropic { .. } => &ANTHROPIC_DEFAULT_HOST,
            ProviderType::OpenAI { .. } => &OPEN_AI_DEFAULT_HOST,
            ProviderType::Generic => unimplemented!(),
        }
    }

    pub fn url(&self) -> &'static Url {
        match self {
            ProviderType::Anthropic { .. } => &ANTHROPIC_DEFAULT_URL,
            ProviderType::OpenAI { .. } => &OPEN_AI_DEFAULT_URL,
            ProviderType::Generic => unimplemented!(),
        }
    }
}

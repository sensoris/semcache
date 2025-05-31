use std::sync::LazyLock;

use axum::http::HeaderValue;
use reqwest::header::ToStrError;
use thiserror::Error;
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

#[derive(Error, Debug)]
pub enum ProviderError {
    #[error("Failed to extract header value: {0}")]
    HeaderParsingError(#[from] ToStrError),
}

pub enum ProviderType {
    Anthropic,
    OpenAI,
    Generic,
}

impl ProviderType {
    pub fn path(&self) -> &'static str {
        match self {
            ProviderType::Anthropic => ANTHROPIC_REST_PATH,
            ProviderType::OpenAI => OPEN_AI_REST_PATH,
            ProviderType::Generic => {
                unimplemented!("No default path exists for the generic provider")
            }
        }
    }
    pub fn prompt_json_path<'request>(
        &self,
        maybe_prompt_location_header: Option<&'request HeaderValue>,
    ) -> Result<&'request str, ProviderError> {
        // if the prompt json path is set in the request, use this
        if let Some(prompt_location_header) = maybe_prompt_location_header {
            return Ok(prompt_location_header.to_str()?);
        };

        // if no json path is set, fall back to defaults per provider
        match self {
            ProviderType::Anthropic => Ok(ANTHROPIC_PROMPT_PATH),
            ProviderType::OpenAI => Ok(OPEN_AI_PROMPT_PATH),
            ProviderType::Generic => todo!(),
        }
    }

    pub fn host_header(&self) -> &'static HeaderValue {
        match self {
            ProviderType::Anthropic => &ANTHROPIC_DEFAULT_HOST,
            ProviderType::OpenAI => &OPEN_AI_DEFAULT_HOST,
            ProviderType::Generic => unimplemented!(),
        }
    }

    pub fn url(&self) -> &'static Url {
        match self {
            ProviderType::Anthropic => &ANTHROPIC_DEFAULT_URL,
            ProviderType::OpenAI => &OPEN_AI_DEFAULT_URL,
            ProviderType::Generic => unimplemented!(),
        }
    }
}

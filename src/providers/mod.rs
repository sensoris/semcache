use std::sync::LazyLock;

use axum::http::HeaderValue;
use reqwest::header::ToStrError;
use thiserror::Error;
use url::{ParseError, Url};

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
static ANTHROPIC_REST_PATH: &str = "/v1/messages";
static OPEN_AI_REST_PATH: &str = "/v1/chat/completions";
static GENERIC_REST_PATH: &str = "/v1/semcache/";

// JSON PROMPT PATH
static ANTHROPIC_PROMPT_PATH: &str = "$.messages[-1].content";
static OPEN_AI_PROMPT_PATH: &str = "$.messages[-1].content";

#[derive(Error, Debug)]
pub enum ProviderError {
    #[error("{0}")]
    StringParsingHeaderError(#[from] ToStrError),
    #[error("{0}")]
    UrlParsingHeaderError(#[from] ParseError),
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
            ProviderType::Generic => GENERIC_REST_PATH,
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

    pub fn url(&self, maybe_upstream_url: Option<&HeaderValue>) -> Result<Url, ProviderError> {
        // if the upstream url is set in the request, use this
        if let Some(upstream_url) = maybe_upstream_url {
            let url_str = upstream_url.to_str()?;
            let parsed_url = Url::parse(url_str)?;
            return Ok(parsed_url);
        };
        match self {
            ProviderType::Anthropic => Ok(ANTHROPIC_DEFAULT_URL.clone()),
            ProviderType::OpenAI => Ok(OPEN_AI_DEFAULT_URL.clone()),
            ProviderType::Generic => unimplemented!(),
        }
    }
}

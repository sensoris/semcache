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
    #[error("Invalid generic provider: {0}")]
    InvalidGenericProvider(String),
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

    pub fn url(
        &self,
        maybe_upstream_url: Option<&HeaderValue>,
        maybe_proxy_host: Option<&HeaderValue>,
    ) -> Result<Url, ProviderError> {
        // if the upstream url is set in the request, use this
        if let Some(upstream_url) = maybe_upstream_url {
            let url_str = upstream_url.to_str()?;
            let parsed_url = Url::parse(url_str)?;
            return Ok(parsed_url);
        }
        // else if you want to override the host but use e.g. OpenAI format
        // this will allow you to call semcache/v1/chat/completions but set X-PROXY-HOST to e.g. "https://api.deepseek.com"
        else if let Some(proxy_host) = maybe_proxy_host {
            let base_url = Url::parse(proxy_host.to_str()?)?;
            return match self {
                ProviderType::Anthropic => Ok(base_url.join(ANTHROPIC_PROMPT_PATH)?),
                ProviderType::OpenAI => Ok(base_url.join(OPEN_AI_PROMPT_PATH)?),
                ProviderType::Generic => Err(ProviderError::InvalidGenericProvider(String::from(
                    "please use the X-LLM_PROXY_UPSREAM header to specify server to forward requests to",
                ))),
            };
        }
        // else go with default for path provided
        match self {
            ProviderType::Anthropic => Ok(ANTHROPIC_DEFAULT_URL.clone()),
            ProviderType::OpenAI => Ok(OPEN_AI_DEFAULT_URL.clone()),
            ProviderType::Generic => unimplemented!(),
        }
    }
}

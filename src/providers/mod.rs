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
            ProviderType::Generic => Err(ProviderError::InvalidGenericProvider(String::from(
                "Please provide a prompt path when using the generic semcache method",
            ))),
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
                ProviderType::Anthropic => Ok(base_url.join(ANTHROPIC_REST_PATH)?),
                ProviderType::OpenAI => Ok(base_url.join(OPEN_AI_REST_PATH)?),
                ProviderType::Generic => Err(ProviderError::InvalidGenericProvider(String::from(
                    "please use the X-LLM-PROXY-UPSTREAM header to specify server to forward requests to",
                ))),
            };
        }
        // else go with default for path provided
        match self {
            ProviderType::Anthropic => Ok(ANTHROPIC_DEFAULT_URL.clone()),
            ProviderType::OpenAI => Ok(OPEN_AI_DEFAULT_URL.clone()),
            ProviderType::Generic => Err(ProviderError::InvalidGenericProvider(String::from(
                "please use the X-LLM-PROXY-UPSTREAM header to specify server to forward requests to",
            ))),
        }
    }
}

#[cfg(test)]
mod tests {

    use axum::http::HeaderValue;
    use url::Url;

    use crate::providers::{
        ANTHROPIC_DEFAULT_URL, ANTHROPIC_PROMPT_PATH, OPEN_AI_DEFAULT_URL, OPEN_AI_PROMPT_PATH,
        ProviderType,
    };

    #[test]
    fn prompt_json_path_openai() {
        // given
        let provider = ProviderType::OpenAI;

        // when
        let path = provider.prompt_json_path(None).unwrap();

        // then
        assert_eq!(path, OPEN_AI_PROMPT_PATH);
    }

    #[test]
    fn prompt_json_path_anthropic() {
        // given
        let provider = ProviderType::Anthropic;

        // when
        let path = provider.prompt_json_path(None).unwrap();

        // then
        assert_eq!(path, ANTHROPIC_PROMPT_PATH);
    }

    #[test]
    fn prompt_json_path_generic_and_path_supplied() {
        // given
        let provider = ProviderType::Generic;

        // when
        let header_prompt = HeaderValue::from_static("$.prompt_path");
        let path = provider.prompt_json_path(Some(&header_prompt)).unwrap();

        // then
        assert_eq!(path, "$.prompt_path");
    }

    #[test]
    fn prompt_json_path_generic_and_no_path_expect_err() {
        // given
        let provider = ProviderType::Generic;

        // when
        let path = provider.prompt_json_path(None);

        // then
        match path {
            Ok(_) => panic!("Should given an error"),
            Err(_) => assert!(true),
        }
    }

    #[test]
    fn url_openai() {
        // given
        let provider = ProviderType::OpenAI;

        // when
        let url = provider.url(None, None).unwrap();

        // then
        assert_eq!(url, *OPEN_AI_DEFAULT_URL);
    }

    #[test]
    fn url_anthropic() {
        // given
        let provider = ProviderType::Anthropic;

        // when
        let url = provider.url(None, None).unwrap();

        // then
        assert_eq!(url, *ANTHROPIC_DEFAULT_URL);
    }

    #[test]
    fn url_openai_when_host_header_provided() {
        // given
        let provider = ProviderType::OpenAI;

        // when
        let url = provider
            .url(
                None,
                Some(&HeaderValue::from_static("https://api.deepseek.com")),
            )
            .unwrap();

        // then
        let expected = Url::parse("https://api.deepseek.com/v1/chat/completions").unwrap();
        assert_eq!(url, expected);
    }

    #[test]
    fn url_openai_when_proxy_upstream_and_host_header_provided() {
        // given
        let provider = ProviderType::OpenAI;
        let proxy_upstream = HeaderValue::from_static("https://clart.com");
        let proxy_host = HeaderValue::from_static("https://api.deepseek.com");

        // when
        let url = provider
            .url(Some(&proxy_upstream), Some(&proxy_host))
            .unwrap();

        // then
        let expected = Url::parse("https://clart.com").unwrap();
        assert_eq!(url, expected);
    }

    #[test]
    fn url_generic_when_proxy_upstream_and_host_header_provided() {
        // given
        let provider = ProviderType::Generic;
        let proxy_upstream = HeaderValue::from_static("https://clart.com");
        let proxy_host = HeaderValue::from_static("https://api.deepseek.com");

        // when
        let url = provider
            .url(Some(&proxy_upstream), Some(&proxy_host))
            .unwrap();

        // then
        let expected = Url::parse("https://clart.com").unwrap();
        assert_eq!(url, expected);
    }

    #[test]
    fn url_generic_when_proxy_upstream_provided() {
        // given
        let provider = ProviderType::Generic;
        let proxy_upstream = HeaderValue::from_static("https://clart.com");

        // when
        let url = provider.url(Some(&proxy_upstream), None).unwrap();

        // then
        let expected = Url::parse("https://clart.com").unwrap();
        assert_eq!(url, expected);
    }

    #[test]
    fn url_generic_when_proxy_host_provided() {
        // given
        let provider = ProviderType::Generic;
        let proxy_host = HeaderValue::from_static("https://api.deepseek.com");

        // when
        let url = provider.url(None, Some(&proxy_host));

        // then
        match url {
            Ok(_) => panic!("Should give an error"),
            Err(_) => assert!(true),
        }
    }

    #[test]
    fn url_generic_when_no_headers_provided() {
        // given
        let provider = ProviderType::Generic;

        // when
        let url = provider.url(None, None);

        // then
        match url {
            Ok(_) => panic!("Should give an error"),
            Err(_) => assert!(true),
        }
    }
}

use super::Provider;
use axum::http::HeaderValue;
use std::sync::LazyLock;
use url::Url;

static DEFAULT_URL: LazyLock<Url> =
    LazyLock::new(|| Url::parse("https://api.anthropic.com/v1/messages").unwrap());

pub struct Anthropic;

impl Provider for Anthropic {
    fn upstream_url(&self) -> &'static Url {
        &DEFAULT_URL
    }

    fn header_host(&self) -> HeaderValue {
        HeaderValue::from_static("api.anthropic.com")
    }

    fn prompt_path(&self) -> &'static str {
        "$.messages[-1].content"
    }

    fn path(&self) -> &'static str {
        "/v1/messages"
    }
}

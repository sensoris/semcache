use super::Provider;
use axum::http::HeaderValue;
use std::sync::LazyLock;
use url::Url;

static DEFAULT_URL: LazyLock<Url> =
    LazyLock::new(|| Url::parse("https://api.openai.com/v1/chat/completions").unwrap());

pub struct Generic;

impl Provider for Generic {
    fn upstream_url(&self) -> &'static Url {
        &DEFAULT_URL
    }

    fn header_host(&self) -> HeaderValue {
        HeaderValue::from_static("api.anthropic.com")
    }

    fn path(&self) -> &'static str {
        "/semcache/chat/completions"
    }

    fn prompt_path(&self) -> &'static str {
        "$.messages[-1].content"
    }
}

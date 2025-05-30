use super::Provider;
use axum::http::HeaderValue;
use std::sync::LazyLock;
use url::Url;

static DEFAULT_URL: LazyLock<Url> =
    LazyLock::new(|| Url::parse("https://api.openai.com/v1/chat/completions").unwrap());

pub struct OpenAI;

impl Provider for OpenAI {
    fn upstream_url(&self) -> &'static Url {
        &DEFAULT_URL
    }

    fn header_host(&self) -> HeaderValue {
        HeaderValue::from_static("api.openai.com")
    }

    fn path(&self) -> &'static str {
        "/v1/chat/completions"
    }

    // We refer to the last message's content as the prompt and key in our cache
    fn prompt_path(&self) -> &'static str {
        "$.messages[-1].content"
    }
}

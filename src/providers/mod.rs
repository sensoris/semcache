use axum::http::HeaderValue;
use url::Url;

pub mod anthropic;
pub mod generic;
pub mod openai;

pub trait Provider {
    // The complete url of the provider's api.
    // E.g. https://api.openai.com/v1/chat/completions
    fn upstream_url(&self) -> &'static Url;

    // The host of the provider's api.
    // E.g. api.openai.com
    fn header_host(&self) -> HeaderValue;

    // The path of the provider's api.
    // E.g /v1/chat/completions
    fn path(&self) -> &'static str;

    // The location of the prompt in the request body in JsonPath format.
    // JsonPath is a query language for JSON. The specification is described in RFC 9535.
    // E.g. $.messages[-1].content
    fn prompt_path(&self) -> &'static str;
}

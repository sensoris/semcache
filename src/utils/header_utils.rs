use std::sync::LazyLock;

use axum::http::{HeaderMap, HeaderName};
use tracing::debug;

use crate::providers::ProviderType;

// HEADERS
pub static PROXY_UPSTREAM_HEADER: HeaderName = HeaderName::from_static("x-llm-proxy-upstream");
pub static PROXY_PROMPT_LOCATION_HEADER: HeaderName = HeaderName::from_static("x-llm-prompt");
pub static HOP_HEADERS: LazyLock<[HeaderName; 10]> = LazyLock::new(|| {
    [
        HeaderName::from_static("connection"),
        HeaderName::from_static("te"),
        HeaderName::from_static("trailer"),
        HeaderName::from_static("keep-alive"),
        HeaderName::from_static("proxy-connection"),
        HeaderName::from_static("proxy-authenticate"),
        HeaderName::from_static("proxy-authorization"),
        HeaderName::from_static("transfer-encoding"),
        HeaderName::from_static("upgrade"),
        // todo - why do we need to remove this?
        HeaderName::from_static("content-length"),
    ]
});

pub fn remove_hop_headers(headers: &mut HeaderMap) {
    debug!("Removing hop headers");

    for header in &*HOP_HEADERS {
        headers.remove(header);
    }
}

pub fn prepare_upstream_headers(headers: HeaderMap, provider: ProviderType) -> HeaderMap {
    let mut upstream_headers = headers.clone();

    remove_hop_headers(&mut upstream_headers);

    // remove semcache headers
    upstream_headers.remove(&PROXY_UPSTREAM_HEADER);
    upstream_headers.remove(&PROXY_PROMPT_LOCATION_HEADER);

    // add host for request to be accepted
    upstream_headers.insert("host", provider.host_header().clone());
    upstream_headers
}

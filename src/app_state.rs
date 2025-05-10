use reqwest::Client;

pub struct AppState {
    pub http_client: Client,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            http_client: Client::new(),
        }
    }
}

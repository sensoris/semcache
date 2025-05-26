use crate::cache::cache::{Cache, EvictionPolicy};
use crate::cache::response_store::ResponseStore;
use crate::cache::semantic_store::flat_ip_faiss_store::FlatIPFaissStore;
use crate::embedding::fastembed::FastEmbedService;
use crate::embedding::service::EmbeddingService;
use reqwest::Client;

pub struct AppState {
    pub http_client: Client,
    pub embedding_service: Box<dyn EmbeddingService>,
    // todo we should probably use base64 or something that isn't string here
    pub cache: Cache<String>,
}

impl AppState {
    pub fn new() -> Self {
        let http_client = Client::new();
        let embedding_service = FastEmbedService::new();
        let semantic_store = FlatIPFaissStore::new(embedding_service.get_dimensionality());
        Self {
            http_client: http_client,
            embedding_service: Box::new(embedding_service),
            cache: Cache::new(
                Box::new(semantic_store),
                ResponseStore::new(),
                0.9,
                EvictionPolicy::EntryLimit(4),
            ),
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::app_state::AppState;

    #[test]
    fn putting_same_prompt_different_response() {
        let app_state = AppState::new();
        let embedding = app_state.embedding_service.embed("First prompt").unwrap();
        
        let first_response = "First response".to_string();
        app_state.cache.put(embedding.clone(), first_response.clone()).expect("TODO: panic message");
        let result = app_state.cache.get_if_present(&embedding).unwrap().expect("a");
        assert_eq!(result, first_response);

        let second_response = "Second response".to_string();
        app_state.cache.put(embedding.clone(), second_response.clone()).expect("TODO: panic message");
        let second_result = app_state.cache.get_if_present(&embedding).unwrap().expect("a");
        assert_eq!(second_result, second_response);
    }
}

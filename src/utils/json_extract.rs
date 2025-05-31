use crate::endpoints::chat::error::CompletionError;
use jsonpath_rust::JsonPath;
use serde_json::Value;

pub fn extract_prompt_from_path(data: &Value, path: &str) -> Result<String, CompletionError> {
    let query_results = data.query_with_path(path)?;

    query_results
        .last()
        .ok_or_else(|| {
            CompletionError::InvalidRequest(format!("No element found at path '{}'", path))
        })
        .and_then(|last_query_ref| {
            let value_ref: &Value = last_query_ref.clone().val();

            value_ref.as_str().map(str::to_owned).ok_or_else(|| {
                CompletionError::InvalidRequest(format!(
                    "Expected a string at path '{}', but found: {:?}",
                    path, value_ref
                ))
            })
        })
}

#[cfg(test)]
mod tests {
    use crate::endpoints::chat::error::CompletionError;
    use crate::utils::json_extract::extract_prompt_from_path;
    use axum::Json;
    use serde_json::json;

    #[test]
    fn test_openai_format_prompt_selection() {
        let body = Json(json!({
            "messages": [
                {"content": "First message!"},
                {"content": "Here is your reply"},
                {"content": "My extremely interesting prompt"}
            ]
        }));

        let result = extract_prompt_from_path(&body, "$.messages[-1].content");

        assert!(result.is_ok());
        let prompt = result.unwrap();
        assert_eq!(prompt, "My extremely interesting prompt");

        let body = Json(json!({
            "messages": [
                {"content": "Single message"},
            ]
        }));

        let result = extract_prompt_from_path(&body, "$.messages[-1].content");

        assert!(result.is_ok());
        let prompt = result.unwrap();
        assert_eq!(prompt, "Single message");
    }

    #[test]
    fn should_return_errors_when_incorrectly_formatted() {
        let body = Json(json!({
            "messages": []
        }));
        let result = extract_prompt_from_path(&body, "$.messages[-1].content");

        match result {
            Err(CompletionError::InvalidRequest(x)) => {
                assert_eq!("No element found at path '$.messages[-1].content'", x)
            }
            _ => panic!("Expected CompletionError::Internal"),
        }

        let body = Json(json!({}));
        let result = extract_prompt_from_path(&body, "$.messages[-1].content");
        match result {
            Err(CompletionError::InvalidRequest(x)) => {
                assert_eq!("No element found at path '$.messages[-1].content'", x)
            }
            _ => panic!("Expected CompletionError::Internal"),
        }
    }

    #[test]
    fn test_simple_prompt_location() {
        let body = Json(json!({
            "prompt": "What is the capital of France?"
        }));

        let result = extract_prompt_from_path(&body, "$.prompt");

        assert!(result.is_ok());
        let prompt = result.unwrap();
        assert_eq!(prompt, "What is the capital of France?");
    }
}

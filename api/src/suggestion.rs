use event_processing::dispatcher::EventDispatcher;
use event_processing::{Event, EventSource, SuggestionEvent};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::error::ApiError;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SuggestionArgs {
    #[schemars(description = "The suggestion or feedback message (max 2000 characters)")]
    pub message: String,
}

pub async fn submit_suggestion(
    args: SuggestionArgs,
    dispatcher: &EventDispatcher,
    source: EventSource,
) -> Result<String, ApiError> {
    let message = args.message.trim().to_string();

    if message.is_empty() {
        return Err(ApiError::Argument(
            "message".to_string(),
            "Suggestion message cannot be empty".to_string(),
        ));
    }

    if message.len() > 2000 {
        return Err(ApiError::Argument(
            "message".to_string(),
            "Suggestion message cannot exceed 2000 characters".to_string(),
        ));
    }

    dispatcher.dispatch(
        source,
        Event::Suggestion(SuggestionEvent { message }),
    );

    Ok("Suggestion received, thank you!".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_submit_suggestion_success() {
        let result = submit_suggestion(
            SuggestionArgs {
                message: "Add year range filtering".to_string(),
            },
            &EventDispatcher::new(),
            EventSource::Web,
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Suggestion received, thank you!");
    }

    #[tokio::test]
    async fn test_submit_suggestion_empty_message() {
        let result = submit_suggestion(
            SuggestionArgs {
                message: "".to_string(),
            },
            &EventDispatcher::new(),
            EventSource::Web,
        )
        .await;

        assert!(matches!(result, Err(ApiError::Argument(ref field, _)) if field == "message"));
    }

    #[tokio::test]
    async fn test_submit_suggestion_whitespace_only() {
        let result = submit_suggestion(
            SuggestionArgs {
                message: "   \n\t  ".to_string(),
            },
            &EventDispatcher::new(),
            EventSource::Web,
        )
        .await;

        assert!(matches!(result, Err(ApiError::Argument(ref field, _)) if field == "message"));
    }

    #[tokio::test]
    async fn test_submit_suggestion_too_long() {
        let long_message = "a".repeat(2001);
        let result = submit_suggestion(
            SuggestionArgs {
                message: long_message,
            },
            &EventDispatcher::new(),
            EventSource::Web,
        )
        .await;

        assert!(matches!(result, Err(ApiError::Argument(ref field, _)) if field == "message"));
    }

    #[tokio::test]
    async fn test_submit_suggestion_trims_whitespace() {
        let result = submit_suggestion(
            SuggestionArgs {
                message: "  Add year range filtering  ".to_string(),
            },
            &EventDispatcher::new(),
            EventSource::Web,
        )
        .await;

        assert!(result.is_ok());
    }
}

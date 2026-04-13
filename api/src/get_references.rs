use std::sync::Arc;

use data_access::metadata::MetadataClient;
use event_processing::dispatcher::EventDispatcher;
use event_processing::{Event, EventSource, GetReferencesEvent};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::error::ApiError;

#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct GetReferencesArgs {
    #[schemars(
        description = "The paper_lyt identifier of the paper (e.g. 'soccer_smallsize__2024__RoboTeam_Twente')"
    )]
    pub paper: String,
}

pub async fn get_references(
    metadata_client: Arc<dyn MetadataClient>,
    args: GetReferencesArgs,
    dispatcher: &EventDispatcher,
    source: EventSource,
) -> Result<Vec<String>, ApiError> {
    let references = metadata_client
        .load_references(args.paper.clone())
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    dispatcher.dispatch(
        source,
        Event::GetReferences(GetReferencesEvent {
            paper: args.paper.clone(),
        }),
    );

    Ok(references)
}

#[cfg(test)]
mod tests {
    use super::*;
    use data_access::metadata::MockMetadataClient;

    #[tokio::test]
    async fn test_get_references() {
        let mut mock = MockMetadataClient::new();

        let expected = vec![
            "Author A. Some paper. 2024.".to_string(),
            "Author B. Another paper. 2023.".to_string(),
        ];
        let expected_clone = expected.clone();

        mock.expect_load_references()
            .withf(|paper_lyt| paper_lyt == "soccer_smallsize__2024__RoboTeam_Twente")
            .returning(move |_| {
                let e = expected_clone.clone();
                Box::pin(std::future::ready(Ok(e)))
            });

        let client = Arc::new(mock);
        let args = GetReferencesArgs {
            paper: "soccer_smallsize__2024__RoboTeam_Twente".to_string(),
        };

        let result = get_references(client, args, &EventDispatcher::new(), EventSource::Web)
            .await
            .unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], "Author A. Some paper. 2024.");
        assert_eq!(result[1], "Author B. Another paper. 2023.");
    }

    #[tokio::test]
    async fn test_get_references_empty() {
        let mut mock = MockMetadataClient::new();

        mock.expect_load_references()
            .withf(|paper_lyt| paper_lyt == "soccer_smallsize__2024__SomeTeam")
            .returning(move |_| {
                Box::pin(std::future::ready(Ok(vec![])))
            });

        let client = Arc::new(mock);
        let args = GetReferencesArgs {
            paper: "soccer_smallsize__2024__SomeTeam".to_string(),
        };

        let result = get_references(client, args, &EventDispatcher::new(), EventSource::Web)
            .await
            .unwrap();

        assert!(result.is_empty());
    }
}

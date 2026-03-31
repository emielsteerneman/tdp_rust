use std::sync::Arc;

use data_access::metadata::MetadataClient;
use event_processing::dispatcher::EventDispatcher;
use event_processing::{Event, EventSource, GetTableOfContentsEvent};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::error::ApiError;

#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct GetTableOfContentsArgs {
    #[schemars(
        description = "The paper_lyt identifier of the paper (e.g. 'soccer_smallsize__2024__RoboTeam_Twente')"
    )]
    pub paper: String,
}

pub async fn get_table_of_contents(
    metadata_client: Arc<dyn MetadataClient>,
    args: GetTableOfContentsArgs,
    dispatcher: &EventDispatcher,
    source: EventSource,
) -> Result<String, ApiError> {
    let toc = metadata_client
        .load_toc(args.paper.clone())
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let mut result = String::from("seq | depth | type  | title\n----|-------|-------|------\n");
    for entry in &toc {
        result.push_str(&format!(
            "{} | {} | {} | {}\n",
            entry.content_seq,
            entry.depth,
            entry.content_type.as_str(),
            entry.title,
        ));
    }

    dispatcher.dispatch(
        source,
        Event::GetTableOfContents(GetTableOfContentsEvent {
            paper: args.paper.clone(),
        }),
    );

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use data_access::metadata::MockMetadataClient;
    use data_structures::content::{ContentType, TocEntry};

    #[tokio::test]
    async fn test_get_table_of_contents() {
        let mut mock = MockMetadataClient::new();

        let entries = vec![
            TocEntry {
                content_seq: 0,
                content_type: ContentType::Text,
                depth: 1,
                title: "Introduction".to_string(),
            },
            TocEntry {
                content_seq: 1,
                content_type: ContentType::Table,
                depth: 2,
                title: "Results Table".to_string(),
            },
        ];
        let entries_clone = entries.clone();

        mock.expect_load_toc()
            .withf(|lyti| lyti == "soccer_smallsize__2024__RoboTeam_Twente")
            .returning(move |_| {
                let e = entries_clone.clone();
                Box::pin(std::future::ready(Ok(e)))
            });

        let client = Arc::new(mock);
        let args = GetTableOfContentsArgs {
            paper: "soccer_smallsize__2024__RoboTeam_Twente".to_string(),
        };

        let result = get_table_of_contents(client, args, &EventDispatcher::new(), EventSource::Web)
            .await
            .unwrap();

        assert!(result.contains("0 | 1 | text | Introduction"));
        assert!(result.contains("1 | 2 | table | Results Table"));
    }
}

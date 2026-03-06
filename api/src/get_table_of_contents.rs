use std::sync::Arc;

use data_access::activity::ActivityClient;
use data_access::metadata::MetadataClient;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::activity::{EventSource, log_activity};
use crate::error::ApiError;

#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct GetTableOfContentsArgs {
    #[schemars(
        description = "The lyti identifier of the paper (e.g. 'soccer_smallsize__2024__RoboTeam_Twente__0')"
    )]
    pub paper: String,
}

pub async fn get_table_of_contents(
    metadata_client: Arc<dyn MetadataClient>,
    args: GetTableOfContentsArgs,
    activity_client: Option<Arc<dyn ActivityClient + Send + Sync>>,
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

    log_activity(
        activity_client,
        source,
        "get_table_of_contents",
        serde_json::json!({
            "paper": args.paper,
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
            .withf(|lyti| lyti == "soccer_smallsize__2024__RoboTeam_Twente__0")
            .returning(move |_| {
                let e = entries_clone.clone();
                Box::pin(std::future::ready(Ok(e)))
            });

        let client = Arc::new(mock);
        let args = GetTableOfContentsArgs {
            paper: "soccer_smallsize__2024__RoboTeam_Twente__0".to_string(),
        };

        let result = get_table_of_contents(client, args, None, EventSource::Dev)
            .await
            .unwrap();

        assert!(result.contains("0 | 1 | text | Introduction"));
        assert!(result.contains("1 | 2 | table | Results Table"));
    }
}

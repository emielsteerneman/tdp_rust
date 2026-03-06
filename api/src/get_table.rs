use std::sync::Arc;

use data_access::activity::ActivityClient;
use data_access::metadata::MetadataClient;
use data_structures::content::ContentType;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::activity::{EventSource, log_activity};
use crate::error::ApiError;

#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct GetTableArgs {
    #[schemars(
        description = "The lyti identifier of the paper (e.g. 'soccer_smallsize__2024__RoboTeam_Twente__0')"
    )]
    pub paper: String,
    #[schemars(description = "The content sequence number from the table of contents")]
    pub content_seq: u32,
}

pub async fn get_table(
    metadata_client: Arc<dyn MetadataClient>,
    args: GetTableArgs,
    activity_client: Option<Arc<dyn ActivityClient + Send + Sync>>,
    source: EventSource,
) -> Result<String, ApiError> {
    let item = metadata_client
        .load_content_item(args.paper.clone(), args.content_seq)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    if item.content_type != ContentType::Table {
        return Err(ApiError::Argument(
            "content_seq".to_string(),
            format!(
                "Expected content type 'table' but found '{}'",
                item.content_type.as_str()
            ),
        ));
    }

    log_activity(
        activity_client,
        source,
        "get_table",
        serde_json::json!({
            "paper": args.paper,
            "content_seq": args.content_seq,
        }),
    );

    Ok(format!("{}\n\n{}", item.title, item.body))
}

#[cfg(test)]
mod tests {
    use super::*;
    use data_access::metadata::MockMetadataClient;
    use data_structures::content::ContentItem;

    #[tokio::test]
    async fn test_get_table() {
        let mut mock = MockMetadataClient::new();

        let item = ContentItem {
            content_seq: 2,
            content_type: ContentType::Table,
            depth: 2,
            title: "Table 1: Results".to_string(),
            body: "col1 | col2\nval1 | val2".to_string(),
            image_path: None,
        };
        let item_clone = item.clone();

        mock.expect_load_content_item()
            .withf(|lyti, seq| lyti == "soccer_smallsize__2024__RoboTeam_Twente__0" && *seq == 2)
            .returning(move |_, _| {
                let i = item_clone.clone();
                Box::pin(std::future::ready(Ok(i)))
            });

        let client = Arc::new(mock);
        let args = GetTableArgs {
            paper: "soccer_smallsize__2024__RoboTeam_Twente__0".to_string(),
            content_seq: 2,
        };

        let result = get_table(client, args, None, EventSource::Dev)
            .await
            .unwrap();

        assert!(result.contains("Table 1: Results"));
        assert!(result.contains("col1 | col2"));
    }

    #[tokio::test]
    async fn test_get_table_wrong_type() {
        let mut mock = MockMetadataClient::new();

        let item = ContentItem {
            content_seq: 0,
            content_type: ContentType::Text,
            depth: 1,
            title: "Introduction".to_string(),
            body: "Some text".to_string(),
            image_path: None,
        };
        let item_clone = item.clone();

        mock.expect_load_content_item()
            .withf(|lyti, seq| lyti == "soccer_smallsize__2024__RoboTeam_Twente__0" && *seq == 0)
            .returning(move |_, _| {
                let i = item_clone.clone();
                Box::pin(std::future::ready(Ok(i)))
            });

        let client = Arc::new(mock);
        let args = GetTableArgs {
            paper: "soccer_smallsize__2024__RoboTeam_Twente__0".to_string(),
            content_seq: 0,
        };

        let result = get_table(client, args, None, EventSource::Dev).await;
        assert!(result.is_err());
    }
}

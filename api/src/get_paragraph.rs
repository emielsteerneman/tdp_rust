use std::sync::Arc;

use data_access::activity::ActivityClient;
use data_access::metadata::MetadataClient;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::activity::{EventSource, log_activity};
use crate::error::ApiError;

#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct GetParagraphArgs {
    #[schemars(
        description = "The lyti identifier of the paper (e.g. 'soccer_smallsize__2024__RoboTeam_Twente__0')"
    )]
    pub paper: String,
    #[schemars(description = "The content sequence number from the table of contents")]
    pub content_seq: u32,
}

pub async fn get_paragraph(
    metadata_client: Arc<dyn MetadataClient>,
    args: GetParagraphArgs,
    activity_client: Option<Arc<dyn ActivityClient + Send + Sync>>,
    source: EventSource,
) -> Result<String, ApiError> {
    let item = metadata_client
        .load_content_item(args.paper.clone(), args.content_seq)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    log_activity(
        activity_client,
        source,
        "get_paragraph",
        serde_json::json!({
            "paper": args.paper,
            "content_seq": args.content_seq,
        }),
    );

    Ok(item.body)
}

#[cfg(test)]
mod tests {
    use super::*;
    use data_access::metadata::MockMetadataClient;
    use data_structures::content::{ContentItem, ContentType};

    #[tokio::test]
    async fn test_get_paragraph() {
        let mut mock = MockMetadataClient::new();

        let item = ContentItem {
            content_seq: 0,
            content_type: ContentType::Text,
            depth: 1,
            title: "Introduction".to_string(),
            body: "This is the introduction text.".to_string(),
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
        let args = GetParagraphArgs {
            paper: "soccer_smallsize__2024__RoboTeam_Twente__0".to_string(),
            content_seq: 0,
        };

        let result = get_paragraph(client, args, None, EventSource::Dev)
            .await
            .unwrap();

        assert_eq!(result, "This is the introduction text.");
    }
}

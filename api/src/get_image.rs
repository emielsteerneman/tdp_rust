use std::sync::Arc;

use data_access::activity::ActivityClient;
use data_access::metadata::MetadataClient;
use data_structures::content::ContentType;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::activity::{EventSource, log_activity};
use crate::error::ApiError;

#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct GetImageArgs {
    #[schemars(
        description = "The lyti identifier of the paper (e.g. 'soccer_smallsize__2024__RoboTeam_Twente__0')"
    )]
    pub paper: String,
    #[schemars(description = "The content sequence number from the table of contents")]
    pub content_seq: u32,
}

pub async fn get_image(
    metadata_client: Arc<dyn MetadataClient>,
    args: GetImageArgs,
    activity_client: Option<Arc<dyn ActivityClient + Send + Sync>>,
    source: EventSource,
) -> Result<String, ApiError> {
    let item = metadata_client
        .load_content_item(args.paper.clone(), args.content_seq)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    if item.content_type != ContentType::Image {
        return Err(ApiError::Argument(
            "content_seq".to_string(),
            format!(
                "Expected content type 'image' but found '{}'",
                item.content_type.as_str()
            ),
        ));
    }

    let path = item.image_path.unwrap_or_default();

    log_activity(
        activity_client,
        source,
        "get_image",
        serde_json::json!({
            "paper": args.paper,
            "content_seq": args.content_seq,
        }),
    );

    Ok(format!("{}\nImage: {}", item.title, path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use data_access::metadata::MockMetadataClient;
    use data_structures::content::ContentItem;

    #[tokio::test]
    async fn test_get_image() {
        let mut mock = MockMetadataClient::new();

        let item = ContentItem {
            content_seq: 1,
            content_type: ContentType::Image,
            depth: 2,
            title: "Figure 1: Robot".to_string(),
            body: String::new(),
            image_path: Some("images/robot.png".to_string()),
        };
        let item_clone = item.clone();

        mock.expect_load_content_item()
            .withf(|lyti, seq| lyti == "soccer_smallsize__2024__RoboTeam_Twente__0" && *seq == 1)
            .returning(move |_, _| {
                let i = item_clone.clone();
                Box::pin(std::future::ready(Ok(i)))
            });

        let client = Arc::new(mock);
        let args = GetImageArgs {
            paper: "soccer_smallsize__2024__RoboTeam_Twente__0".to_string(),
            content_seq: 1,
        };

        let result = get_image(client, args, None, EventSource::Dev)
            .await
            .unwrap();

        assert!(result.contains("Figure 1: Robot"));
        assert!(result.contains("Image: images/robot.png"));
    }

    #[tokio::test]
    async fn test_get_image_wrong_type() {
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
        let args = GetImageArgs {
            paper: "soccer_smallsize__2024__RoboTeam_Twente__0".to_string(),
            content_seq: 0,
        };

        let result = get_image(client, args, None, EventSource::Dev).await;
        assert!(result.is_err());
    }
}

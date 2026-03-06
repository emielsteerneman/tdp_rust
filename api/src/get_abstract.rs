use std::sync::Arc;

use data_access::activity::ActivityClient;
use data_access::metadata::MetadataClient;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::activity::{EventSource, log_activity};
use crate::error::ApiError;

#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct GetAbstractArgs {
    #[schemars(
        description = "The lyti identifier of the paper (e.g. 'soccer_smallsize__2024__RoboTeam_Twente__0')"
    )]
    pub paper: String,
}

pub async fn get_abstract(
    metadata_client: Arc<dyn MetadataClient>,
    args: GetAbstractArgs,
    activity_client: Option<Arc<dyn ActivityClient + Send + Sync>>,
    source: EventSource,
) -> Result<String, ApiError> {
    let abstract_text = metadata_client
        .load_paper_abstract(args.paper.clone())
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    log_activity(
        activity_client,
        source,
        "get_abstract",
        serde_json::json!({
            "paper": args.paper,
        }),
    );

    Ok(abstract_text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use data_access::metadata::MockMetadataClient;

    #[tokio::test]
    async fn test_get_abstract() {
        let mut mock = MockMetadataClient::new();

        let expected = "This paper presents our robot system for RoboCup 2024.".to_string();
        let expected_clone = expected.clone();

        mock.expect_load_paper_abstract()
            .withf(|lyti| lyti == "soccer_smallsize__2024__RoboTeam_Twente__0")
            .returning(move |_| {
                let e = expected_clone.clone();
                Box::pin(std::future::ready(Ok(e)))
            });

        let client = Arc::new(mock);
        let args = GetAbstractArgs {
            paper: "soccer_smallsize__2024__RoboTeam_Twente__0".to_string(),
        };

        let result = get_abstract(client, args, None, EventSource::Dev)
            .await
            .unwrap();

        assert_eq!(result, "This paper presents our robot system for RoboCup 2024.");
    }
}

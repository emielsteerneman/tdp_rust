use std::sync::Arc;

use data_access::activity::ActivityClient;
use data_access::metadata::MetadataClient;
use data_structures::file::League;

use crate::activity::{EventSource, log_activity};
use crate::error::ApiError;

pub async fn list_leagues(
    metadata_client: Arc<dyn MetadataClient>,
    activity_client: Option<Arc<dyn ActivityClient + Send + Sync>>,
    source: EventSource,
) -> Result<Vec<League>, ApiError> {
    let leagues = metadata_client
        .load_leagues()
        .await
        .map_err(|err| ApiError::Internal(err.to_string()))?;

    log_activity(
        activity_client,
        source,
        "list_leagues",
        serde_json::json!({
            "result_count": leagues.len(),
        }),
    );

    Ok(leagues)
}

#[cfg(test)]
mod tests {
    use data_access::metadata::MockMetadataClient;
    use data_structures::file::League;
    use std::sync::Arc;

    use super::list_leagues;
    use crate::activity::EventSource;

    #[tokio::test]
    async fn test_list_leagues() -> Result<(), Box<dyn std::error::Error>> {
        let mut client = MockMetadataClient::new();

        client.expect_load_leagues().returning(|| {
            Box::pin(async move {
                Ok(vec![
                    League::new("soccer".to_string(), "smallsize".to_string(), None),
                    League::new("soccer".to_string(), "midsize".to_string(), None),
                    League::new("rescue".to_string(), "simulation".to_string(), None),
                ])
            })
        });

        let client = Arc::new(client);

        let leagues = list_leagues(client.clone(), None, EventSource::Web).await?;
        assert_eq!(leagues.len(), 3);
        assert_eq!(leagues[0].name, "soccer_smallsize");
        assert_eq!(leagues[1].name, "soccer_midsize");
        assert_eq!(leagues[2].name, "rescue_simulation");

        Ok(())
    }
}

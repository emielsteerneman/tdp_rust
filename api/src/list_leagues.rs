use std::sync::Arc;

use data_access::metadata::MetadataClient;
use data_structures::file::League;
use event_processing::dispatcher::EventDispatcher;
use event_processing::{Event, EventSource, ListLeaguesEvent};

use crate::error::ApiError;

pub async fn list_leagues(
    metadata_client: Arc<dyn MetadataClient>,
    dispatcher: &EventDispatcher,
    source: EventSource,
) -> Result<Vec<League>, ApiError> {
    let leagues = metadata_client
        .load_leagues()
        .await
        .map_err(|err| ApiError::Internal(err.to_string()))?;

    dispatcher.dispatch(
        source,
        Event::ListLeagues(ListLeaguesEvent {
            result_count: leagues.len(),
        }),
    );

    Ok(leagues)
}

#[cfg(test)]
mod tests {
    use data_access::metadata::MockMetadataClient;
    use data_structures::file::League;
    use event_processing::dispatcher::EventDispatcher;
    use event_processing::EventSource;
    use std::sync::Arc;

    use super::list_leagues;

    #[tokio::test]
    async fn test_list_leagues() -> Result<(), Box<dyn std::error::Error>> {
        let mut client = MockMetadataClient::new();

        client.expect_load_leagues().returning(|| {
            Box::pin(async move {
                Ok(vec![
                    League::SoccerSmallSize,
                    League::SoccerMidSize,
                    League::RescueRobot,
                ])
            })
        });

        let client = Arc::new(client);

        let leagues = list_leagues(client.clone(), &EventDispatcher::new(), EventSource::Web).await?;
        assert_eq!(leagues.len(), 3);
        assert_eq!(leagues[0].name(), "soccer_smallsize");
        assert_eq!(leagues[1].name(), "soccer_midsize");
        assert_eq!(leagues[2].name(), "rescue_robot");

        Ok(())
    }
}

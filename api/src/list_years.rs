use std::collections::BTreeSet;
use std::sync::Arc;

use data_access::metadata::MetadataClient;
use event_processing::dispatcher::EventDispatcher;
use event_processing::{Event, EventSource, ListYearsEvent};

use crate::error::ApiError;
use crate::paper_filter::PaperFilter;

pub async fn list_years(
    metadata_client: Arc<dyn MetadataClient>,
    filter: PaperFilter,
    dispatcher: &EventDispatcher,
    source: EventSource,
) -> Result<Vec<u32>, ApiError> {
    let papers = metadata_client
        .load_tdps()
        .await
        .map_err(|err| ApiError::Internal(err.to_string()))?;

    let filtered = filter.filter_papers(papers)?;
    let years: BTreeSet<u32> = filtered.iter().map(|tdp| tdp.year).collect();
    let result: Vec<u32> = years.into_iter().collect();

    dispatcher.dispatch(
        source,
        Event::ListYears(ListYearsEvent {
            league: filter.league.clone(),
            team: filter.team.clone(),
            result_count: result.len(),
        }),
    );

    Ok(result)
}

#[cfg(test)]
mod tests {
    use data_access::metadata::MockMetadataClient;
    use data_structures::file::{League, TDPName, TeamName};
    use std::sync::Arc;

    use super::list_years;
    use event_processing::dispatcher::EventDispatcher;
    use event_processing::EventSource;
    use crate::paper_filter::PaperFilter;

    #[tokio::test]
    async fn test_list_years() -> Result<(), Box<dyn std::error::Error>> {
        let mut client = MockMetadataClient::new();

        client.expect_load_tdps().returning(|| {
            Box::pin(async move {
                Ok(vec![
                    TDPName::new(
                        League::SoccerSmallSize,
                        2019,
                        TeamName::from_pretty("RoboTeam Twente"),
                        None,
                    ),
                    TDPName::new(
                        League::SoccerSmallSize,
                        2020,
                        TeamName::from_pretty("Er-Force"),
                        None,
                    ),
                    TDPName::new(
                        League::SoccerMidSize,
                        2019,
                        TeamName::from_pretty("TIGERs Mannheim"),
                        None,
                    ),
                    TDPName::new(
                        League::SoccerMidSize,
                        2021,
                        TeamName::from_pretty("Delft Mercurians"),
                        None,
                    ),
                ])
            })
        });

        let client = Arc::new(client);

        let years = list_years(client.clone(), PaperFilter::default(), &EventDispatcher::new(), EventSource::Web).await?;
        assert_eq!(years.len(), 3);
        assert_eq!(years, vec![2019, 2020, 2021]);

        Ok(())
    }
}

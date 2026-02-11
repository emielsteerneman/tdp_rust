use std::sync::Arc;

use data_access::metadata::MetadataClient;
use data_structures::file::TDPName;

use crate::error::ApiError;

pub async fn list_papers(
    metadata_client: Arc<dyn MetadataClient>,
) -> Result<Vec<TDPName>, ApiError> {
    let papers = metadata_client
        .load_tdps()
        .await
        .map_err(|err| ApiError::Internal(err.to_string()))?;

    Ok(papers)
}

#[cfg(test)]
mod tests {
    use data_access::metadata::MockMetadataClient;
    use data_structures::file::{League, TDPName, TeamName};
    use std::sync::Arc;

    use super::list_papers;

    #[tokio::test]
    async fn test_list_papers() -> Result<(), Box<dyn std::error::Error>> {
        let mut client = MockMetadataClient::new();

        client.expect_load_tdps().returning(|| {
            Box::pin(async move {
                Ok(vec![
                    TDPName::new(
                        League::new("soccer".to_string(), "smallsize".to_string(), None),
                        2019,
                        TeamName::from_pretty("RoboTeam Twente"),
                        None,
                    ),
                    TDPName::new(
                        League::new("soccer".to_string(), "smallsize".to_string(), None),
                        2020,
                        TeamName::from_pretty("Er-Force"),
                        None,
                    ),
                    TDPName::new(
                        League::new("soccer".to_string(), "midsize".to_string(), None),
                        2019,
                        TeamName::from_pretty("TIGERs Mannheim"),
                        None,
                    ),
                ])
            })
        });

        let client = Arc::new(client);

        let papers = list_papers(client.clone()).await?;
        assert_eq!(papers.len(), 3);
        assert_eq!(papers[0].team_name.name_pretty, "RoboTeam Twente");
        assert_eq!(papers[0].year, 2019);
        assert_eq!(papers[1].team_name.name_pretty, "Er-Force");
        assert_eq!(papers[1].year, 2020);
        assert_eq!(papers[2].team_name.name_pretty, "TIGERs Mannheim");
        assert_eq!(papers[2].year, 2019);

        Ok(())
    }
}

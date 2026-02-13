use std::sync::Arc;

use data_access::activity::ActivityClient;
use data_access::metadata::MetadataClient;
use data_processing::text::match_terms;
use data_structures::file::TeamName;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::activity::{EventSource, log_activity};
use crate::error::ApiError;

#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct ListTeamsArgs {
    #[schemars(description = "Optional search term or partial name to filter the list of teams.")]
    pub hint: Option<String>,
}

pub async fn list_teams(
    metadata_client: Arc<dyn MetadataClient>,
    args: ListTeamsArgs,
    activity_client: Option<Arc<dyn ActivityClient + Send + Sync>>,
    source: EventSource,
) -> Result<Vec<TeamName>, ApiError> {
    let mut teams = metadata_client
        .load_teams()
        .await
        .map_err(|err| ApiError::Internal(err.to_string()))?;

    if let Some(hint) = &args.hint {
        let team_names = teams.iter().map(Into::into).collect();
        let matches = match_terms(team_names, hint.clone(), Some(0.8));
        teams = matches
            .iter()
            .map(|team_name| TeamName::new(team_name))
            .collect();
    }

    log_activity(
        activity_client,
        source,
        "list_teams",
        serde_json::json!({
            "hint": args.hint,
            "result_count": teams.len(),
        }),
    );

    Ok(teams)
}

#[cfg(test)]
mod tests {
    use data_access::metadata::MockMetadataClient;
    use data_structures::file::TeamName;
    use std::sync::Arc;

    use super::{ListTeamsArgs, list_teams};
    use crate::activity::EventSource;

    #[tokio::test]
    async fn test_list_teams() -> Result<(), Box<dyn std::error::Error>> {
        let mut client = MockMetadataClient::new();

        client.expect_load_teams().returning(|| {
            Box::pin(async move {
                Ok(vec![
                    TeamName::from_pretty("RoboTeam Twente"),
                    TeamName::from_pretty("Er-Force"),
                    TeamName::from_pretty("TIGERs Mannheim"),
                    TeamName::from_pretty("Delft Mercurians"),
                    TeamName::from_pretty("RoboDragons"),
                ])
            })
        });

        let client = Arc::new(client);

        let teams = list_teams(
            client.clone(),
            ListTeamsArgs { hint: None },
            None,
            EventSource::Web,
        )
        .await?;
        assert_eq!(teams.len(), 5);

        let teams = list_teams(
            client.clone(),
            ListTeamsArgs {
                hint: Some("robo".to_string()),
            },
            None,
            EventSource::Web,
        )
        .await?;
        println!("Received teams: {teams:?}");
        assert_eq!(teams.len(), 2);
        assert!(matches!(
            teams[0].name_pretty.as_str(),
            "RoboTeam Twente" | "RoboDragons"
        ));
        assert!(matches!(
            teams[1].name_pretty.as_str(),
            "RoboTeam Twente" | "RoboDragons"
        ));

        Ok(())
    }
}

use std::sync::Arc;

use data_access::metadata::MetadataClient;
use data_processing::text::match_terms;
use data_structures::file::TeamName;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::tools::ToolError;

#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct ListTeamsArgs {
    #[schemars(description = "Optional search term or partial name to filter the list of teams.")]
    pub hint: Option<String>,
}

pub async fn list_teams(
    metadata_client: Arc<dyn MetadataClient>,
    args: ListTeamsArgs,
) -> Result<Vec<TeamName>, ToolError> {
    let mut teams = metadata_client
        .load_teams()
        .await
        .map_err(|err| ToolError::Internal(err.to_string()))?;

    if let Some(hint) = args.hint {
        let team_names = teams.iter().map(Into::into).collect();
        let matches = match_terms(team_names, hint, Some(0.8));
        teams = matches
            .iter()
            .map(|team_name| TeamName::new(team_name))
            .collect();
    }

    Ok(teams)
}

#[cfg(test)]
mod tests {
    use data_access::metadata::MockMetadataClient;
    use data_structures::file::TeamName;
    use std::sync::Arc;

    use crate::tools::list_teams::{ListTeamsArgs, list_teams};

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

        let teams = list_teams(client.clone(), ListTeamsArgs { hint: None }).await?;
        assert_eq!(teams.len(), 5);

        let teams = list_teams(
            client.clone(),
            ListTeamsArgs {
                hint: Some("robo".to_string()),
            },
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

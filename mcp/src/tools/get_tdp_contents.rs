use std::sync::Arc;

use data_access::metadata::MetadataClient;
use data_structures::file::{League, TDPName, TeamName};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::tools::ToolError;

#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct GetTdpContentsArgs {
    #[schemars(
        description = "The league to which the tdp belongs. For example 'Soccer Smallsize'"
    )]
    pub league: String,
    #[schemars(description = "The year in which the tdp was written. For example 2025")]
    pub year: u32,
    #[schemars(description = "The team who wrote the tdp. For example 'RoboTeam Twente'")]
    pub team: String,
}

pub async fn get_tdp_contents(
    metadata_client: Arc<dyn MetadataClient>,
    args: GetTdpContentsArgs,
) -> Result<String, ToolError> {
    let league = League::try_from(args.league.as_str())
        .map_err(|e| ToolError::Argument("league".to_string(), e.to_string()))?;
    let team_name = TeamName::new(&args.team);
    let tdp_name = TDPName::new(league, args.year, team_name, None);

    let markdown = metadata_client
        .get_tdp_markdown(tdp_name)
        .await
        .map_err(|e| ToolError::Internal(e.to_string()))?;

    Ok(markdown)
}

#[cfg(test)]
mod tests {
    use super::*;
    use data_access::metadata::MockMetadataClient;
    use mockall::predicate;

    #[tokio::test]
    async fn test_get_tdp_contents() {
        let mut mock = MockMetadataClient::new();

        let expected_markdown = "# TDP Content";
        let markdown_clone = expected_markdown.to_string();

        mock.expect_get_tdp_markdown()
            .with(predicate::function(|tdp: &TDPName| {
                tdp.league.name_pretty == "Soccer SmallSize"
                    && tdp.year == 2019
                    && tdp.team_name.name_pretty == "RoboTeam Twente"
            }))
            .returning(move |_| Box::pin(std::future::ready(Ok(markdown_clone.clone()))));

        let client = Arc::new(mock);
        let args = GetTdpContentsArgs {
            league: "soccer_smallsize".to_string(),
            year: 2019,
            team: "RoboTeam Twente".to_string(),
        };

        let result = get_tdp_contents(client, args).await.unwrap();
        assert_eq!(result, expected_markdown);
    }
}
use std::sync::Arc;

use data_access::teams::{TeamMetadataEntry, TeamRegistryClient};
use data_structures::file::TeamName;
use event_processing::dispatcher::EventDispatcher;
use event_processing::{Event, EventSource, GetTeamInfoEvent};
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetTeamInfoArgs {
    #[schemars(description = "Team name (e.g. 'TIGERs Mannheim' or 'TIGERs_Mannheim')")]
    pub team: String,
}

pub async fn get_team_info(
    team_registry: Arc<dyn TeamRegistryClient + Send + Sync>,
    args: GetTeamInfoArgs,
    dispatcher: &EventDispatcher,
    source: EventSource,
) -> anyhow::Result<Vec<TeamMetadataEntry>> {
    let team_name = TeamName::new(args.team.trim());

    let entries = team_registry
        .get_team_metadata(&team_name.name)
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    dispatcher.dispatch(
        source,
        Event::GetTeamInfo(GetTeamInfoEvent {
            team: team_name.name.clone(),
        }),
    );

    Ok(entries)
}

use std::sync::Arc;

use data_access::registry::{RegistryEntry, RegistryClient};
use data_structures::file::League;
use event_processing::dispatcher::EventDispatcher;
use event_processing::{Event, EventSource, GetLeagueInfoEvent};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::error::ApiError;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetLeagueInfoArgs {
    #[schemars(description = "League name (e.g. 'Soccer SmallSize' or 'soccer_smallsize')")]
    pub league: String,
}

pub async fn get_league_info(
    registry: Arc<dyn RegistryClient + Send + Sync>,
    args: GetLeagueInfoArgs,
    dispatcher: &EventDispatcher,
    source: EventSource,
) -> Result<Vec<RegistryEntry>, ApiError> {
    let league = League::try_from(args.league.trim())
        .map_err(|_| ApiError::Argument("league".to_string(), format!("Unknown league: '{}'", args.league)))?;

    let entries = registry
        .get_league_metadata(league.name())
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    dispatcher.dispatch(
        source,
        Event::GetLeagueInfo(GetLeagueInfoEvent {
            league: league.name().to_string(),
        }),
    );

    Ok(entries)
}

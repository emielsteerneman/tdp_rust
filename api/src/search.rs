use std::sync::Arc;

use data_access::activity::ActivityClient;
use data_processing::search::Searcher;
use data_structures::{
    embed_type::EmbedType,
    file::{League, LeagueParseError, TeamName},
    filter::Filter,
};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::activity::{EventSource, log_activity};

#[derive(thiserror::Error, Debug)]
pub enum SearchError {
    #[error("Failed to parse league: {0}")]
    LeagueParseError(#[from] LeagueParseError),
    #[error("Failed to parse year: {0}")]
    YearParseError(#[from] std::num::ParseIntError),
}

#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct SearchArgs {
    #[schemars(
        description = "The search query. Use short keyword phrases like 'trajectory planning', 'omnidirectional drive', or 'ball detection neural network'. Avoid full sentences."
    )]
    pub query: String,
    #[schemars(description = "Maximum number of result chunks to return. Defaults to 10 if not specified.")]
    pub limit: Option<u64>,

    #[schemars(
        description = "Optional comma-separated league filter. Use exact league names from list_leagues, e.g. 'Soccer SmallSize, Soccer Humanoid AdultSize'. Omit to search across all leagues."
    )]
    pub league_filter: Option<String>,

    #[schemars(
        description = "Optional comma-separated year filter, e.g. '2023, 2024'. Useful for finding recent innovations or tracking a topic over time."
    )]
    pub year_filter: Option<String>,

    #[schemars(
        description = "Optional comma-separated team filter. Use exact team names from list_teams, e.g. 'RoboTeam Twente, TIGERs Mannheim'. Useful when you know which team works on a topic."
    )]
    pub team_filter: Option<String>,

    #[schemars(
        description = "Optional comma-separated filter for specific papers by their league__year__team__index identifier, e.g. 'soccer_smallsize__2024__RoboTeam_Twente__0'. Rarely needed — prefer league/year/team filters."
    )]
    pub lyti_filter: Option<String>,

    #[schemars(
        description = "Search method: 'hybrid' (default, best for most queries — combines semantic and keyword matching), 'sparse' (keyword-only, best for exact technical terms like 'PID controller'), 'dense' (semantic-only, best for conceptual queries like 'how to make robots kick harder')."
    )]
    pub search_type: EmbedType,
}

impl SearchArgs {
    pub fn to_filter(&self) -> Result<Option<Filter>, SearchError> {
        let mut filter = Filter::default();

        if let Some(league_filter) = &self.league_filter {
            for league in league_filter.split(",") {
                filter.add_league(League::try_from(league.trim())?);
            }
        }

        if let Some(year_filter) = &self.year_filter {
            for year in year_filter.split(",") {
                filter.add_year(year.trim().parse()?);
            }
        }

        if let Some(team_filter) = &self.team_filter {
            for team in team_filter.split(",") {
                filter.add_team(TeamName::from_pretty(team.trim()));
            }
        }

        if let Some(lyti_filter) = &self.lyti_filter {
            for lyti in lyti_filter.split(",") {
                filter.add_league_year_team_index(lyti.trim().to_string());
            }
        }

        Ok(Some(filter))
    }
}

pub async fn search(
    searcher: &Searcher,
    args: SearchArgs,
    activity_client: Option<Arc<dyn ActivityClient + Send + Sync>>,
    source: EventSource,
) -> anyhow::Result<String> {
    let search_type_str = format!("{:?}", args.search_type);
    let search_result = searcher
        .search(
            args.query.clone(),
            args.limit,
            args.to_filter()?,
            args.search_type.into(),
        )
        .await?;

    log_activity(
        activity_client,
        source,
        "search",
        serde_json::json!({
            "query": args.query,
            "search_type": search_type_str,
            "result_count": search_result.chunks.len(),
            "league_filter": args.league_filter,
            "year_filter": args.year_filter,
            "team_filter": args.team_filter,
        }),
    );

    Ok(serde_json::to_string_pretty(&search_result)?)
}

pub async fn search_structured(
    searcher: &Searcher,
    args: SearchArgs,
    activity_client: Option<Arc<dyn ActivityClient + Send + Sync>>,
    source: EventSource,
) -> anyhow::Result<data_structures::intermediate::SearchResult> {
    let search_type_str = format!("{:?}", args.search_type);
    let search_result = searcher
        .search(
            args.query.clone(),
            args.limit,
            args.to_filter()?,
            args.search_type.into(),
        )
        .await?;

    log_activity(
        activity_client,
        source,
        "search",
        serde_json::json!({
            "query": args.query,
            "search_type": search_type_str,
            "result_count": search_result.chunks.len(),
            "league_filter": args.league_filter,
            "year_filter": args.year_filter,
            "team_filter": args.team_filter,
        }),
    );

    Ok(search_result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_searchargs_to_filter() {
        let args = SearchArgs {
            query: "test".to_string(),
            limit: Some(10),
            league_filter: Some("Soccer Smallsize, Soccer Humanoid".to_string()),
            year_filter: Some("2021, 2024".to_string()),
            team_filter: Some("RoboTeam Twente, TIGERs Mannheim".to_string()),
            lyti_filter: Some("rescue_simulation_infrastructure__2012__UvA_Rescue__0".to_string()),
            search_type: EmbedType::DENSE,
        };

        let filter = args.to_filter().unwrap().unwrap();

        assert!(
            filter
                .leagues
                .as_ref()
                .unwrap()
                .contains("Soccer SmallSize")
        );
        assert!(filter.leagues.as_ref().unwrap().contains("Soccer Humanoid"));
        assert!(filter.years.as_ref().unwrap().contains(&2021));
        assert!(filter.years.as_ref().unwrap().contains(&2024));
        assert!(filter.teams.as_ref().unwrap().contains("RoboTeam Twente"));
        assert!(filter.teams.as_ref().unwrap().contains("TIGERs Mannheim"));
        assert!(
            filter
                .league_year_team_indexes
                .as_ref()
                .unwrap()
                .contains("rescue_simulation_infrastructure__2012__UvA_Rescue__0")
        );
    }

    #[test]
    fn test_searchargs_to_filter_errors() {
        // Test invalid year
        let args = SearchArgs {
            year_filter: Some("2021, not_a_year".to_string()),
            ..Default::default()
        };
        let result = args.to_filter();
        assert!(matches!(result, Err(SearchError::YearParseError(_))));

        // Test invalid league separator
        let args = SearchArgs {
            league_filter: Some("InvalidLeague".to_string()),
            ..Default::default()
        };
        let result = args.to_filter();
        assert!(matches!(
            result,
            Err(SearchError::LeagueParseError(
                LeagueParseError::BadSeparator()
            ))
        ));

        // Test invalid league field count
        let args = SearchArgs {
            league_filter: Some("soccer_smallsize_extra_field".to_string()),
            ..Default::default()
        };
        let result = args.to_filter();
        assert!(matches!(
            result,
            Err(SearchError::LeagueParseError(
                LeagueParseError::BadFieldCount(4)
            ))
        ));
    }
}

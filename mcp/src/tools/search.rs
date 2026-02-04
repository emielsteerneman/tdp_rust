use crate::state::AppState;
use data_structures::{
    embed_type::EmbedType,
    file::{League, LeagueParseError, TeamName},
    filter::Filter,
};
use rmcp::schemars::JsonSchema;
use serde::Deserialize;

#[derive(thiserror::Error, Debug)]
pub enum SearchError {
    #[error("Failed to parse league: {0}")]
    LeagueParseError(#[from] LeagueParseError),
    #[error("Failed to parse year: {0}")]
    YearParseError(#[from] std::num::ParseIntError),
}

#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct SearchArgs {
    #[schemars(description = "The query for which to search. For example 'battery capacity'.")]
    pub query: String,
    #[schemars(description = "Limit the number of results.")]
    pub limit: Option<u64>,

    #[schemars(
        description = "An optional comma separated filter over the leagues. For example 'Soccer Smallsize, Soccer Humanoid Adult'"
    )]
    pub league_filter: Option<String>,

    #[schemars(
        description = "An optional comma separated filter over the years. For example '2021, 2024, 2025'"
    )]
    pub year_filter: Option<String>,

    #[schemars(
        description = "An optional comma separated filter over the teams. For example 'RoboTeam Twente, TIGERs Mannheim, Er-Force'"
    )]
    pub team_filter: Option<String>,

    #[schemars(
        description = "An optional comma separated filter over the year_league_team_index (so, specific papers). For example 'rescue_simulation_infrastructure__2012__UvA_Rescue__0, rescue_robot__2019__MRL__0'"
    )]
    pub lyti_filter: Option<String>,

    #[schemars(
        description = "Indicates whether to search using dense semantic embeddings, sparse keyword embeddings, or a hybrid of both. Possible values: dense, sparse, hybrid"
    )]
    pub search_type: EmbedType,
}

impl SearchArgs {
    fn to_filter(&self) -> Result<Option<Filter>, SearchError> {
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

pub async fn search(state: &AppState, args: SearchArgs) -> anyhow::Result<String> {
    let search_result = state
        .searcher
        .search(
            args.query.clone(),
            args.limit,
            args.to_filter()?,
            args.search_type.into(),
        )
        .await?;

    Ok(serde_json::to_string_pretty(&search_result)?)
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

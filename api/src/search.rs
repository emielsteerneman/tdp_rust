use data_processing::search::Searcher;
use data_structures::{
    embed_type::EmbedType,
    file::{League, LeagueParseError, TeamName},
    filter::Filter,
};
use event_processing::dispatcher::EventDispatcher;
use event_processing::{Event, EventSource, SearchEvent};
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(thiserror::Error, Debug)]
pub enum SearchError {
    #[error("Failed to parse league: {0}")]
    LeagueParseError(#[from] LeagueParseError),
    #[error("Failed to parse year: {0}")]
    YearParseError(#[from] std::num::ParseIntError),
    #[error("Invalid content type: '{0}'. Use 'text', 'table', or 'image'.")]
    ContentTypeParseError(String),
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
        description = "Optional comma-separated filter for specific papers by their paper_lyt identifier, e.g. 'soccer_smallsize__2024__RoboTeam_Twente'. Rarely needed — prefer league/year/team filters."
    )]
    pub paper_lyt_filter: Option<String>,

    #[schemars(
        description = "Optional comma-separated content type filter. Values: 'text', 'table', 'image'. E.g. 'text, table' to exclude images. Defaults to all types."
    )]
    pub content_type_filter: Option<String>,

    #[schemars(
        description = "Search method: 'hybrid' (default, best for most queries — combines semantic and keyword matching), 'sparse' (keyword-only, best for exact technical terms like 'PID controller'), 'dense' (semantic-only, best for conceptual queries like 'how to make robots kick harder')."
    )]
    pub search_type: Option<EmbedType>,
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
                filter.add_team(TeamName::new(team.trim()));
            }
        }

        if let Some(paper_lyt_filter) = &self.paper_lyt_filter {
            for paper_lyt in paper_lyt_filter.split(",") {
                filter.add_paper_lyt(paper_lyt.trim().to_string());
            }
        }

        if let Some(content_type_filter) = &self.content_type_filter {
            for ct in content_type_filter.split(",") {
                let ct = ct.trim().to_lowercase();
                if !["text", "table", "image"].contains(&ct.as_str()) {
                    return Err(SearchError::ContentTypeParseError(ct));
                }
                filter.add_content_type(ct);
            }
        }

        Ok(Some(filter))
    }
}

pub async fn search(
    searcher: &Searcher,
    args: SearchArgs,
    dispatcher: &EventDispatcher,
    source: EventSource,
) -> anyhow::Result<data_structures::intermediate::SearchResult> {
    let search_type = args.search_type.unwrap_or_default();
    let search_type_str = match search_type {
        EmbedType::DENSE => "dense",
        EmbedType::SPARSE => "sparse",
        EmbedType::HYBRID => "hybrid",
    };
    let search_result = searcher
        .search(
            args.query.clone(),
            args.limit,
            args.to_filter()?,
            search_type.into(),
        )
        .await?;

    dispatcher.dispatch(
        source,
        Event::Search(SearchEvent {
            query: args.query.clone(),
            search_type: search_type_str.to_string(),
            result_count: search_result.chunks.len(),
            league_filter: args.league_filter.clone(),
            year_filter: args.year_filter.clone(),
            team_filter: args.team_filter.clone(),
            content_type_filter: args.content_type_filter.clone(),
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
            league_filter: Some("Soccer SmallSize, Soccer MidSize".to_string()),
            year_filter: Some("2021, 2024".to_string()),
            team_filter: Some("RoboTeam Twente, TIGERs Mannheim".to_string()),
            paper_lyt_filter: Some("rescue_simulation_infrastructure__2012__UvA_Rescue".to_string()),
            content_type_filter: Some("text, table".to_string()),
            search_type: Some(EmbedType::DENSE),
        };

        let filter = args.to_filter().unwrap().unwrap();

        assert!(
            filter
                .leagues
                .as_ref()
                .unwrap()
                .contains(&League::SoccerSmallSize)
        );
        assert!(filter.leagues.as_ref().unwrap().contains(&League::SoccerMidSize));
        assert!(filter.years.as_ref().unwrap().contains(&2021));
        assert!(filter.years.as_ref().unwrap().contains(&2024));
        assert!(filter.teams.as_ref().unwrap().contains("RoboTeam_Twente"));
        assert!(filter.teams.as_ref().unwrap().contains("TIGERs_Mannheim"));
        assert!(
            filter
                .paper_lyts
                .as_ref()
                .unwrap()
                .contains("rescue_simulation_infrastructure__2012__UvA_Rescue")
        );
        assert!(filter.content_types.as_ref().unwrap().contains("text"));
        assert!(filter.content_types.as_ref().unwrap().contains("table"));
        assert!(!filter.content_types.as_ref().unwrap().contains("image"));
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

        // Test invalid league name
        let args = SearchArgs {
            league_filter: Some("InvalidLeague".to_string()),
            ..Default::default()
        };
        let result = args.to_filter();
        assert!(matches!(
            result,
            Err(SearchError::LeagueParseError(
                LeagueParseError::Unknown(_)
            ))
        ));

        // Test another invalid league name (was previously BadFieldCount)
        let args = SearchArgs {
            league_filter: Some("soccer_smallsize_extra_field".to_string()),
            ..Default::default()
        };
        let result = args.to_filter();
        assert!(matches!(
            result,
            Err(SearchError::LeagueParseError(
                LeagueParseError::Unknown(_)
            ))
        ));

        // Test invalid content type
        let args = SearchArgs {
            content_type_filter: Some("text, video".to_string()),
            ..Default::default()
        };
        let result = args.to_filter();
        assert!(matches!(
            result,
            Err(SearchError::ContentTypeParseError(_))
        ));
    }
}

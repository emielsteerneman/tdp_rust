use data_structures::file::{League, TDPName, TeamName};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::error::ApiError;

#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct PaperFilter {
    #[schemars(
        description = "Optional league filter, e.g. 'Soccer SmallSize'. Only return results from this league."
    )]
    pub league: Option<String>,
    #[schemars(description = "Optional year filter, e.g. 2024. Only return results from this year.")]
    pub year: Option<u32>,
    #[schemars(
        description = "Optional team filter, e.g. 'TIGERs Mannheim'. Only return results from this team."
    )]
    pub team: Option<String>,
}

impl PaperFilter {
    pub fn matches(&self, paper: &TDPName) -> Result<bool, ApiError> {
        if let Some(ref league_str) = self.league {
            let league = League::try_from(league_str.as_str())
                .map_err(|e| ApiError::Argument("league".to_string(), e.to_string()))?;
            if paper.league.name != league.name {
                return Ok(false);
            }
        }
        if let Some(year) = self.year {
            if paper.year != year {
                return Ok(false);
            }
        }
        if let Some(ref team_str) = self.team {
            let team = TeamName::from_pretty(team_str);
            if paper.team_name.name != team.name {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub fn filter_papers(&self, papers: Vec<TDPName>) -> Result<Vec<TDPName>, ApiError> {
        papers
            .into_iter()
            .filter_map(|p| match self.matches(&p) {
                Ok(true) => Some(Ok(p)),
                Ok(false) => None,
                Err(e) => Some(Err(e)),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use data_structures::file::League;

    fn test_papers() -> Vec<TDPName> {
        vec![
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
        ]
    }

    #[test]
    fn test_no_filter() {
        let filter = PaperFilter::default();
        let result = filter.filter_papers(test_papers()).unwrap();
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_league_filter() {
        let filter = PaperFilter {
            league: Some("Soccer SmallSize".to_string()),
            ..Default::default()
        };
        let result = filter.filter_papers(test_papers()).unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_year_filter() {
        let filter = PaperFilter {
            year: Some(2019),
            ..Default::default()
        };
        let result = filter.filter_papers(test_papers()).unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_team_filter() {
        let filter = PaperFilter {
            team: Some("TIGERs Mannheim".to_string()),
            ..Default::default()
        };
        let result = filter.filter_papers(test_papers()).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].team_name.name_pretty, "TIGERs Mannheim");
    }

    #[test]
    fn test_combined_filters() {
        let filter = PaperFilter {
            league: Some("Soccer SmallSize".to_string()),
            year: Some(2019),
            ..Default::default()
        };
        let result = filter.filter_papers(test_papers()).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].team_name.name_pretty, "RoboTeam Twente");
    }

    #[test]
    fn test_invalid_league() {
        let filter = PaperFilter {
            league: Some("InvalidLeague".to_string()),
            ..Default::default()
        };
        assert!(filter.filter_papers(test_papers()).is_err());
    }
}

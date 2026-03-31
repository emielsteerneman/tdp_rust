use std::collections::HashSet;

use crate::file::{League, TDPName, TeamName};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Filter {
    #[schemars(description = "An optional list of team names on which to filter results")]
    pub teams: Option<HashSet<String>>,
    #[schemars(description = "An optional list of leagues on which to filter results")]
    pub leagues: Option<HashSet<League>>,
    #[schemars(description = "An optional list of years on which to filter results")]
    pub years: Option<HashSet<u32>>,
    #[schemars(
        description = "An optional list of paper_lyt identifiers on which to filter results"
    )]
    pub paper_lyts: Option<HashSet<String>>,
    #[schemars(
        description = "An optional list of content types on which to filter results (text, table, image)"
    )]
    pub content_types: Option<HashSet<String>>,
}

impl Filter {
    pub fn add_team(&mut self, team: TeamName) {
        let teams = self.teams.get_or_insert_with(HashSet::new);
        teams.insert(team.name);
    }

    pub fn add_league(&mut self, league: League) {
        let leagues = self.leagues.get_or_insert_with(HashSet::new);
        leagues.insert(league);
    }

    pub fn add_year(&mut self, year: u32) {
        let years = self.years.get_or_insert_with(HashSet::new);
        years.insert(year);
    }

    pub fn add_tdp(&mut self, tdp_name: TDPName) {
        self.add_paper_lyt(tdp_name.get_paper_lyt());
    }

    pub fn add_paper_lyt(&mut self, paper_lyt: String) {
        let paper_lyts = self.paper_lyts.get_or_insert_with(HashSet::new);
        paper_lyts.insert(paper_lyt);
    }

    pub fn add_content_type(&mut self, content_type: String) {
        let content_types = self.content_types.get_or_insert_with(HashSet::new);
        content_types.insert(content_type);
    }

    pub fn matches_tdp_name(&self, tdp_name: &TDPName) -> bool {
        if let Some(teams) = &self.teams {
            if !teams.contains(&tdp_name.team_name.name) {
                return false;
            }
        }
        if let Some(leagues) = &self.leagues {
            if !leagues.contains(&tdp_name.league) {
                return false;
            }
        }
        if let Some(years) = &self.years {
            if !years.contains(&tdp_name.year) {
                return false;
            }
        }
        if let Some(paper_lyts) = &self.paper_lyts {
            if !paper_lyts.contains(&tdp_name.get_paper_lyt()) {
                return false;
            }
        }
        true
    }
}

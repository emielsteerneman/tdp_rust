use std::collections::HashSet;

use crate::file::{League, TDPName, TeamName};
use crate::intermediate::Chunk;
use crate::paper::TDP;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Filter {
    #[schemars(description = "An optional list of team names on which to filter results")]
    pub teams: Option<HashSet<String>>,
    #[schemars(description = "An optional list of leagues on which to filter results")]
    pub leagues: Option<HashSet<String>>,
    #[schemars(description = "An optional list of years on which to filter results")]
    pub years: Option<HashSet<u32>>,
    #[schemars(
        description = "An optional list of league_year_team_index on which to filter results"
    )]
    pub league_year_team_indexes: Option<HashSet<String>>,
}

impl Filter {
    pub fn add_team(&mut self, team: TeamName) {
        let teams = self.teams.get_or_insert_with(HashSet::new);
        teams.insert(team.name_pretty);
    }

    pub fn add_league(&mut self, league: League) {
        let leagues = self.leagues.get_or_insert_with(HashSet::new);
        leagues.insert(league.name_pretty);
    }

    pub fn add_year(&mut self, year: u32) {
        let years = self.years.get_or_insert_with(HashSet::new);
        years.insert(year);
    }

    pub fn add_tdp(&mut self, tdp_name: TDPName) {
        self.add_league_year_team_index(tdp_name.get_filename());
    }

    pub fn add_chunk(&mut self, chunk: Chunk) {
        self.add_league_year_team_index(chunk.league_year_team_idx);
    }

    pub fn add_league_year_team_index(&mut self, league_year_team_index: String) {
        let league_year_team_indexes = self
            .league_year_team_indexes
            .get_or_insert_with(HashSet::new);
        league_year_team_indexes.insert(league_year_team_index);
    }

    pub fn matches_chunk(&self, chunk: &Chunk) -> bool {
        if let Some(teams) = &self.teams {
            if !teams.contains(&chunk.team.name) {
                return false;
            }
        }
        if let Some(leagues) = &self.leagues {
            if !leagues.contains(&chunk.league.name) {
                return false;
            }
        }
        if let Some(years) = &self.years {
            if !years.contains(&chunk.year) {
                return false;
            }
        }
        if let Some(indexes) = &self.league_year_team_indexes {
            if !indexes.contains(&chunk.league_year_team_idx) {
                return false;
            }
        }
        true
    }

    pub fn matches_tdp_name(&self, tdp_name: &TDPName) -> bool {
        if let Some(teams) = &self.teams {
            if !teams.contains(&tdp_name.team_name.name_pretty) {
                return false;
            }
        }
        if let Some(leagues) = &self.leagues {
            if !leagues.contains(&tdp_name.league.name_pretty) {
                return false;
            }
        }
        if let Some(years) = &self.years {
            if !years.contains(&tdp_name.year) {
                return false;
            }
        }
        if let Some(indexes) = &self.league_year_team_indexes {
            if !indexes.contains(&tdp_name.get_filename()) {
                return false;
            }
        }
        true
    }

    pub fn matches_tdp(&self, tdp: &TDP) -> bool {
        self.matches_tdp_name(&tdp.name)
    }
}

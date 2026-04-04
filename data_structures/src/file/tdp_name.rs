use serde::{Deserialize, Serialize};

use crate::file::{League, LeagueParseError, TeamName};

#[derive(thiserror::Error, Debug)]
pub enum TDPParseError {
    #[error("expected 3 fields separated by '__', got {0}")]
    BadFieldCount(usize),
    #[error("Could not parse league: {0}")]
    League(#[from] LeagueParseError),
    #[error("invalid team name: {0}")]
    Team(String),
    #[error("invalid year: {0}")]
    Year(String),
    #[error("missing file stem in path")]
    NoFileStem,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TDPName {
    pub league: League,
    pub team_name: TeamName,
    pub year: u32,
}

impl TDPName {
    pub const PDF_EXT: &'static str = ".pdf";
    pub const HTML_EXT: &'static str = ".html";

    pub fn new(league: League, year: u32, team_name: TeamName) -> Self {
        Self {
            league,
            team_name,
            year,
        }
    }

    pub fn get_paper_lyt(&self) -> String {
        format!("{}__{}__{}", self.league.name(), self.year, self.team_name.name)
    }
}

impl TryFrom<&str> for TDPName {
    type Error = TDPParseError;

    fn try_from(string: &str) -> Result<Self, TDPParseError> {
        // strip extension if present
        let base = match string.rsplit_once('.') {
            Some((stem, _ext)) => stem,
            None => &string,
        };

        let parts: Vec<&str> = base.split("__").collect();
        if parts.len() != 3 {
            return Err(TDPParseError::BadFieldCount(parts.len()));
        }

        let l = parts[0];
        let y = parts[1];
        let t = parts[2];

        let league: League = l.try_into()?;
        let year: u32 = y.parse().map_err(|_| TDPParseError::Year(y.to_string()))?;
        let team_name: TeamName = TeamName::new(t);

        Ok(Self::new(league, year, team_name))
    }
}

#[cfg(test)]
mod tests {
    use crate::file::{League, TDPName};

    #[test]
    pub fn test_basic() {
        let filename = "soccer_smallsize__2019__RoboTeam_Twente.pdf";
        let tdp_name: TDPName = filename.try_into().unwrap();

        assert_eq!(tdp_name.league, League::SoccerSmallSize);
        assert_eq!(tdp_name.league.name_pretty(), "Soccer SmallSize");
        assert_eq!(tdp_name.year, 2019);
        assert_eq!(tdp_name.team_name.name, "RoboTeam_Twente");
        assert_eq!(tdp_name.team_name.name_pretty, "RoboTeam Twente");
    }

    #[test]
    pub fn test_deserialize() {
        let json = r#"{"league": "industrial_atwork", "team_name": {"name": "Carologistics", "name_pretty": "Carologistics"}, "year": 2019}"#;

        let tdp_name: TDPName = serde_json::from_str(json).unwrap();

        assert_eq!(tdp_name.league, League::IndustrialAtwork);
        assert_eq!(tdp_name.league.name_pretty(), "Industrial @Work");
        assert_eq!(tdp_name.year, 2019);
        assert_eq!(tdp_name.team_name.name, "Carologistics");
        assert_eq!(tdp_name.team_name.name_pretty, "Carologistics");
    }
}

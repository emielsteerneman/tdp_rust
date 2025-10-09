use serde::Deserialize;

use crate::file::{League, LeagueParseError, TeamName};

#[derive(thiserror::Error, Debug)]
pub enum TDPParseError {
    #[error("expected 4 fields separated by '__', got {0}")]
    BadFieldCount(usize),
    #[error("Could not parse league: {0}")]
    League(#[from] LeagueParseError),
    #[error("invalid team name: {0}")]
    Team(String),
    #[error("invalid year: {0}")]
    Year(String),
    #[error("invalid index: {0}")]
    Index(String),
    #[error("missing file stem in path")]
    NoFileStem,
}

#[derive(Clone, Deserialize)]
pub struct TDPName {
    pub league: League,
    pub team_name: TeamName,
    pub year: u32,
    pub index: u32,
}

impl TDPName {
    pub const PDF_EXT: &'static str = ".pdf";
    pub const HTML_EXT: &'static str = ".html";

    pub fn new(league: League, year: u32, team_name: TeamName, index: Option<u32>) -> Self {
        let index = index.unwrap_or(0);

        Self {
            league,
            team_name,
            year,
            index,
        }
    }

    pub fn get_filename(&self) -> String {
        let filename = format!(
            "{}__{}__{}__{}",
            self.league.name, self.year, self.team_name.name, self.index
        );
        filename
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
        if parts.len() != 4 {
            return Err(TDPParseError::BadFieldCount(parts.len()));
        }

        let l = parts[0];
        let y = parts[1];
        let t = parts[2];
        let i = parts[3];

        let league: League = l.try_into()?;
        let year: u32 = y.parse().map_err(|_| TDPParseError::Year(y.to_string()))?;
        let team_name: TeamName = TeamName::new(t);
        let index: u32 = i.parse().map_err(|_| TDPParseError::Index(i.to_string()))?;

        Ok(Self::new(league, year, team_name, Some(index)))
    }
}

#[cfg(test)]
mod tests {
    use crate::file::TDPName;

    #[test]
    pub fn test_basic() {
        let filename = "soccer_smallsize__2019__RoboTeam_Twente__1.pdf";
        let tdp_name: TDPName = filename.try_into().unwrap();

        assert_eq!(tdp_name.league.name_pretty, "Soccer SmallSize");
        assert_eq!(tdp_name.year, 2019);
        assert_eq!(tdp_name.team_name.name_pretty, "RoboTeam Twente");
        assert_eq!(tdp_name.index, 1);
    }

    #[test]
    pub fn test_deserialize() {
        let json = r#"{"league": {"league_major": "industrial", "league_minor": "logistics", "league_sub": null, "name": "industrial_logistics", "name_pretty": "Industrial Logistics"}, "team_name": {"name": "Carologistics", "name_pretty": "Carologistics"}, "year": 2019, "index": 0}"#;

        let tdp_name: TDPName = serde_json::from_str(json).unwrap();

        println!("{}", tdp_name.league.name_pretty);
        println!("{}", tdp_name.year);
        println!("{}", tdp_name.team_name.name);
        println!("{}", tdp_name.index);

        assert_eq!(tdp_name.league.name_pretty, "Industrial Logistics");
        assert_eq!(tdp_name.year, 2019);
        assert_eq!(tdp_name.team_name.name_pretty, "Carologistics");
        assert_eq!(tdp_name.index, 0);
    }
}

use crate::file::{League, LeagueParseError, TeamName};

#[derive(Clone)]
pub struct TDPName {
    pub league: League,
    pub team_name: TeamName,
    pub year: u32,
    pub index: u32,
    pub filename: String,
}

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

impl TDPName {
    pub const PDF_EXT: &'static str = ".pdf";
    pub const HTML_EXT: &'static str = ".html";

    pub fn new(league: League, year: u32, team_name: TeamName, index: Option<u32>) -> Self {
        let index = index.unwrap_or(0);
        let filename = format!("{}__{}__{}__{}", league.name, year, team_name.name, index);

        Self {
            league,
            team_name,
            year,
            index,
            filename,
        }
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
}

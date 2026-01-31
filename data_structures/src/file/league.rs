use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct League {
    pub league_major: String,
    pub league_minor: String,
    pub league_sub: Option<String>,
    pub name: String,
    pub name_pretty: String,
}

#[derive(thiserror::Error, Debug)]
pub enum LeagueParseError {
    #[error("expected 2 or 3 fields separated by '_', got {0}")]
    BadFieldCount(usize),
    #[error("expected either ' ' or '_' as separator")]
    BadSeparator(),
}

impl League {
    pub fn new(league_major: String, league_minor: String, league_sub: Option<String>) -> Self {
        let name = match &league_sub {
            Some(sub) => format!(
                "{}_{}_{}",
                league_major.to_lowercase(),
                league_minor.to_lowercase(),
                sub.to_lowercase()
            ),
            None => format!(
                "{}_{}",
                league_major.to_lowercase(),
                league_minor.to_lowercase()
            ),
        };

        let name_pretty = name_to_name_pretty(name.clone());

        Self {
            league_major,
            league_minor,
            league_sub,
            name,
            name_pretty,
        }
    }
}

impl Default for League {
    fn default() -> Self {
        Self {
            league_major: "soccer".to_string(),
            league_minor: "smallsize".to_string(),
            league_sub: None,
            name: "soccer_smallsize".to_string(),
            name_pretty: "Soccer SmallSize".to_string(),
        }
    }
}

impl TryFrom<&str> for League {
    type Error = LeagueParseError;

    fn try_from(value: &str) -> Result<Self, LeagueParseError> {
        let separator = if value.contains(" ") {
            Ok(" ")
        } else if value.contains("_") {
            Ok("_")
        } else {
            Err(LeagueParseError::BadSeparator())
        }?;

        let parts = value.split(separator).collect::<Vec<_>>();
        match parts.as_slice() {
            [major, minor] => Ok(Self::new((*major).to_string(), (*minor).to_string(), None)),
            [major, minor, sub] => Ok(Self::new(
                (*major).to_string(),
                (*minor).to_string(),
                Some((*sub).to_string()),
            )),
            _ => Err(LeagueParseError::BadFieldCount(parts.len())),
        }
    }
}

impl Into<String> for League {
    fn into(self) -> String {
        self.name
    }
}

impl Into<String> for &League {
    fn into(self) -> String {
        self.name.clone()
    }
}

fn name_to_name_pretty(name: String) -> String {
    let mut name_pretty = name.replace('_', " ");
    name_pretty = capitalize_words(&name_pretty);

    for (from, to) in [
        ("Smallsize", "SmallSize"),
        ("Midsize", "MidSize"),
        ("Standardplatform", "StandardPlatform"),
        ("Atwork", "@Work"),
        ("Athome", "@Home"),
        ("2d", "2D"),
        ("3d", "3D"),
    ] {
        name_pretty = name_pretty.replace(from, to);
    }

    name_pretty
}

fn capitalize_words(s: &str) -> String {
    s.split_whitespace()
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                Some(first) => first
                    .to_uppercase()
                    .chain(chars.flat_map(|c| c.to_lowercase()))
                    .collect(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use crate::file::League;

    #[test]
    fn test_from_string() -> Result<(), Box<dyn std::error::Error>> {
        let league: League = "Test League".try_into()?;
        assert_eq!(league.name, "test_league");
        assert_eq!(league.name_pretty, "Test League");

        let league: League = "test_league".try_into()?;
        assert_eq!(league.name, "test_league");
        assert_eq!(league.name_pretty, "Test League");

        let league: League = "Test League Sub".try_into()?;
        assert_eq!(league.name, "test_league_sub");
        assert_eq!(league.name_pretty, "Test League Sub");

        let league: League = "test_league_sub".try_into()?;
        assert_eq!(league.name, "test_league_sub");
        assert_eq!(league.name_pretty, "Test League Sub");

        Ok(())
    }
}

use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct League {
    pub league_major: String,
    pub league_minor: String,
    pub league_sub: Option<String>,
    pub name: String,
    pub name_pretty: String,
}

#[derive(thiserror::Error, Debug)]
pub enum LeagueParseError {
    #[error("expected 4 fields separated by '__', got {0}")]
    BadFieldCount(usize),
}

impl League {
    pub fn new(league_major: String, league_minor: String, league_sub: Option<String>) -> Self {
        let name = match &league_sub {
            Some(sub) => format!("{}_{}_{}", league_major, league_minor, sub),
            None => format!("{}_{}", league_major, league_minor),
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

impl TryFrom<&str> for League {
    type Error = LeagueParseError;

    fn try_from(value: &str) -> Result<Self, LeagueParseError> {
        let parts: Vec<&str> = value.split('_').collect();
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

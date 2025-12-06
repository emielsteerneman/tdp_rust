use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TeamName {
    pub name: String,
    pub name_pretty: String,
}

impl TeamName {
    pub fn new(name: &str) -> Self {
        let name = name.to_string();
        let name_pretty = name.replace("_", " ");

        TeamName { name, name_pretty }
    }

    pub fn from_pretty(name_pretty: &str) -> Self {
        let name_pretty = name_pretty.to_string();
        let name = name_pretty.replace(" ", "_");

        TeamName { name, name_pretty }
    }
}

impl Into<String> for TeamName {
    fn into(self) -> String {
        self.name
    }
}

impl Into<String> for &TeamName {
    fn into(self) -> String {
        self.name.clone()
    }
}

#[cfg(test)]
mod tests {
    use crate::file::TDPName;

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

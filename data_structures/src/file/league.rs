use std::borrow::Cow;
use std::fmt;

use schemars::JsonSchema;
use serde::de::{self, MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum League {
    SoccerSmallSize,
    SoccerMidSize,
    SoccerStandardPlatform,
    SoccerHumanoidAdult,
    SoccerHumanoidKid,
    SoccerHumanoidTeen,
    SoccerSimulation2D,
    SoccerSimulation3D,
    RescueRobot,
    RescueSimulationAgent,
    RescueSimulationInfrastructure,
    RescueSimulationVirtual,
    AthomeDomestic,
    AthomeOpen,
    AthomeSocial,
    IndustrialAtwork,
}

#[derive(thiserror::Error, Debug)]
pub enum LeagueParseError {
    #[error("unknown league: '{0}'")]
    Unknown(String),
}

/// All 16 leagues with their (variant, machine_name, pretty_name) for lookup.
const LEAGUE_TABLE: &[(League, &str, &str)] = &[
    (League::SoccerSmallSize, "soccer_smallsize", "Soccer SmallSize"),
    (League::SoccerMidSize, "soccer_midsize", "Soccer MidSize"),
    (League::SoccerStandardPlatform, "soccer_standardplatform", "Soccer StandardPlatform"),
    (League::SoccerHumanoidAdult, "soccer_humanoid_adult", "Soccer Humanoid Adult"),
    (League::SoccerHumanoidKid, "soccer_humanoid_kid", "Soccer Humanoid Kid"),
    (League::SoccerHumanoidTeen, "soccer_humanoid_teen", "Soccer Humanoid Teen"),
    (League::SoccerSimulation2D, "soccer_simulation_2d", "Soccer Simulation 2D"),
    (League::SoccerSimulation3D, "soccer_simulation_3d", "Soccer Simulation 3D"),
    (League::RescueRobot, "rescue_robot", "Rescue Robot"),
    (League::RescueSimulationAgent, "rescue_simulation_agent", "Rescue Simulation Agent"),
    (League::RescueSimulationInfrastructure, "rescue_simulation_infrastructure", "Rescue Simulation Infrastructure"),
    (League::RescueSimulationVirtual, "rescue_simulation_virtual", "Rescue Simulation Virtual"),
    (League::AthomeDomestic, "athome_domestic", "@Home Domestic"),
    (League::AthomeOpen, "athome_open", "@Home Open"),
    (League::AthomeSocial, "athome_social", "@Home Social"),
    (League::IndustrialAtwork, "industrial_atwork", "Industrial @Work"),
];

impl League {
    pub fn all() -> &'static [League] {
        static ALL: [League; 16] = [
            League::SoccerSmallSize, League::SoccerMidSize, League::SoccerStandardPlatform,
            League::SoccerHumanoidAdult, League::SoccerHumanoidKid, League::SoccerHumanoidTeen,
            League::SoccerSimulation2D, League::SoccerSimulation3D,
            League::RescueRobot, League::RescueSimulationAgent,
            League::RescueSimulationInfrastructure, League::RescueSimulationVirtual,
            League::AthomeDomestic, League::AthomeOpen, League::AthomeSocial,
            League::IndustrialAtwork,
        ];
        &ALL
    }

    pub fn name(&self) -> &'static str {
        LEAGUE_TABLE.iter().find(|(l, _, _)| l == self).unwrap().1
    }

    pub fn name_pretty(&self) -> &'static str {
        LEAGUE_TABLE.iter().find(|(l, _, _)| l == self).unwrap().2
    }

    pub fn major(&self) -> &'static str {
        match self {
            League::SoccerSmallSize | League::SoccerMidSize | League::SoccerStandardPlatform
            | League::SoccerHumanoidAdult | League::SoccerHumanoidKid | League::SoccerHumanoidTeen
            | League::SoccerSimulation2D | League::SoccerSimulation3D => "soccer",
            League::RescueRobot | League::RescueSimulationAgent
            | League::RescueSimulationInfrastructure | League::RescueSimulationVirtual => "rescue",
            League::AthomeDomestic | League::AthomeOpen | League::AthomeSocial => "athome",
            League::IndustrialAtwork => "industrial",
        }
    }

    pub fn major_pretty(&self) -> &'static str {
        match self {
            League::SoccerSmallSize | League::SoccerMidSize | League::SoccerStandardPlatform
            | League::SoccerHumanoidAdult | League::SoccerHumanoidKid | League::SoccerHumanoidTeen
            | League::SoccerSimulation2D | League::SoccerSimulation3D => "Soccer",
            League::RescueRobot | League::RescueSimulationAgent
            | League::RescueSimulationInfrastructure | League::RescueSimulationVirtual => "Rescue",
            League::AthomeDomestic | League::AthomeOpen | League::AthomeSocial => "@Home",
            League::IndustrialAtwork => "Industrial",
        }
    }

    pub fn minor(&self) -> &'static str {
        match self {
            League::SoccerSmallSize => "smallsize",
            League::SoccerMidSize => "midsize",
            League::SoccerStandardPlatform => "standardplatform",
            League::SoccerHumanoidAdult | League::SoccerHumanoidKid | League::SoccerHumanoidTeen => "humanoid",
            League::SoccerSimulation2D | League::SoccerSimulation3D => "simulation",
            League::RescueRobot => "robot",
            League::RescueSimulationAgent | League::RescueSimulationInfrastructure | League::RescueSimulationVirtual => "simulation",
            League::AthomeDomestic => "domestic",
            League::AthomeOpen => "open",
            League::AthomeSocial => "social",
            League::IndustrialAtwork => "atwork",
        }
    }

    pub fn minor_pretty(&self) -> &'static str {
        match self {
            League::SoccerSmallSize => "SmallSize",
            League::SoccerMidSize => "MidSize",
            League::SoccerStandardPlatform => "StandardPlatform",
            League::SoccerHumanoidAdult | League::SoccerHumanoidKid | League::SoccerHumanoidTeen => "Humanoid",
            League::SoccerSimulation2D | League::SoccerSimulation3D => "Simulation",
            League::RescueRobot => "Robot",
            League::RescueSimulationAgent | League::RescueSimulationInfrastructure | League::RescueSimulationVirtual => "Simulation",
            League::AthomeDomestic => "Domestic",
            League::AthomeOpen => "Open",
            League::AthomeSocial => "Social",
            League::IndustrialAtwork => "@Work",
        }
    }

    pub fn sub(&self) -> Option<&'static str> {
        match self {
            League::SoccerHumanoidAdult => Some("adult"),
            League::SoccerHumanoidKid => Some("kid"),
            League::SoccerHumanoidTeen => Some("teen"),
            League::SoccerSimulation2D => Some("2d"),
            League::SoccerSimulation3D => Some("3d"),
            League::RescueSimulationAgent => Some("agent"),
            League::RescueSimulationInfrastructure => Some("infrastructure"),
            League::RescueSimulationVirtual => Some("virtual"),
            _ => None,
        }
    }

    pub fn sub_pretty(&self) -> Option<&'static str> {
        match self {
            League::SoccerHumanoidAdult => Some("Adult"),
            League::SoccerHumanoidKid => Some("Kid"),
            League::SoccerHumanoidTeen => Some("Teen"),
            League::SoccerSimulation2D => Some("2D"),
            League::SoccerSimulation3D => Some("3D"),
            League::RescueSimulationAgent => Some("Agent"),
            League::RescueSimulationInfrastructure => Some("Infrastructure"),
            League::RescueSimulationVirtual => Some("Virtual"),
            _ => None,
        }
    }
}

impl TryFrom<&str> for League {
    type Error = LeagueParseError;

    fn try_from(value: &str) -> Result<Self, LeagueParseError> {
        let value = value.trim();
        for &(league, machine, pretty) in LEAGUE_TABLE {
            if value == machine || value == pretty {
                return Ok(league);
            }
        }
        Err(LeagueParseError::Unknown(value.to_string()))
    }
}

impl fmt::Display for League {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name_pretty())
    }
}

// --- Serde ---

impl Serialize for League {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(5))?;
        map.serialize_entry("league_major", self.major_pretty())?;
        map.serialize_entry("league_minor", self.minor_pretty())?;
        map.serialize_entry("league_sub", &self.sub_pretty())?;
        map.serialize_entry("name", self.name())?;
        map.serialize_entry("name_pretty", self.name_pretty())?;
        map.end()
    }
}

impl<'de> Deserialize<'de> for League {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct LeagueVisitor;

        impl<'de> Visitor<'de> for LeagueVisitor {
            type Value = League;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a league name string or league object with 'name' field")
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<League, E> {
                League::try_from(v).map_err(de::Error::custom)
            }

            fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<League, M::Error> {
                let mut name: Option<String> = None;
                while let Some(key) = map.next_key::<String>()? {
                    if key == "name" {
                        name = Some(map.next_value()?);
                    } else {
                        let _: serde_json::Value = map.next_value()?;
                    }
                }
                let name = name.ok_or_else(|| de::Error::missing_field("name"))?;
                League::try_from(name.as_str()).map_err(de::Error::custom)
            }
        }

        deserializer.deserialize_any(LeagueVisitor)
    }
}

// --- JsonSchema (schemars 1.x API — uses json_schema! macro) ---

impl JsonSchema for League {
    fn schema_name() -> Cow<'static, str> {
        "League".into()
    }

    fn inline_schema() -> bool {
        true
    }

    fn json_schema(_generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
            "type": "object",
            "properties": {
                "name": { "type": "string" },
                "name_pretty": { "type": "string" },
                "league_major": { "type": "string" },
                "league_minor": { "type": "string" },
                "league_sub": { "type": ["string", "null"] }
            },
            "required": ["name", "name_pretty", "league_major", "league_minor"]
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_variants_name_roundtrip() {
        let all = League::all();
        assert_eq!(all.len(), 16);
        for league in all {
            // name() -> try_from -> same variant
            let parsed = League::try_from(league.name()).unwrap();
            assert_eq!(parsed, *league);
            // name_pretty() -> try_from -> same variant
            let parsed = League::try_from(league.name_pretty()).unwrap();
            assert_eq!(parsed, *league);
        }
    }

    #[test]
    fn test_specific_names() {
        assert_eq!(League::SoccerSmallSize.name(), "soccer_smallsize");
        assert_eq!(League::SoccerSmallSize.name_pretty(), "Soccer SmallSize");
        assert_eq!(League::IndustrialAtwork.name(), "industrial_atwork");
        assert_eq!(League::IndustrialAtwork.name_pretty(), "Industrial @Work");
        assert_eq!(League::AthomeDomestic.name(), "athome_domestic");
        assert_eq!(League::AthomeDomestic.name_pretty(), "@Home Domestic");
        assert_eq!(League::SoccerSimulation2D.name(), "soccer_simulation_2d");
        assert_eq!(League::SoccerSimulation2D.name_pretty(), "Soccer Simulation 2D");
    }

    #[test]
    fn test_hierarchy() {
        assert_eq!(League::SoccerSmallSize.major(), "soccer");
        assert_eq!(League::SoccerSmallSize.major_pretty(), "Soccer");
        assert_eq!(League::SoccerSmallSize.minor(), "smallsize");
        assert_eq!(League::SoccerSmallSize.minor_pretty(), "SmallSize");
        assert_eq!(League::SoccerSmallSize.sub(), None);

        assert_eq!(League::SoccerHumanoidAdult.major(), "soccer");
        assert_eq!(League::SoccerHumanoidAdult.minor(), "humanoid");
        assert_eq!(League::SoccerHumanoidAdult.minor_pretty(), "Humanoid");
        assert_eq!(League::SoccerHumanoidAdult.sub(), Some("adult"));
        assert_eq!(League::SoccerHumanoidAdult.sub_pretty(), Some("Adult"));

        assert_eq!(League::AthomeDomestic.major(), "athome");
        assert_eq!(League::AthomeDomestic.major_pretty(), "@Home");
        assert_eq!(League::IndustrialAtwork.minor(), "atwork");
        assert_eq!(League::IndustrialAtwork.minor_pretty(), "@Work");
    }

    #[test]
    fn test_try_from_machine_names() {
        assert_eq!(League::try_from("soccer_smallsize").unwrap(), League::SoccerSmallSize);
        assert_eq!(League::try_from("soccer_humanoid_adult").unwrap(), League::SoccerHumanoidAdult);
        assert_eq!(League::try_from("industrial_atwork").unwrap(), League::IndustrialAtwork);
        assert_eq!(League::try_from("athome_domestic").unwrap(), League::AthomeDomestic);
    }

    #[test]
    fn test_try_from_pretty_names() {
        assert_eq!(League::try_from("Soccer SmallSize").unwrap(), League::SoccerSmallSize);
        assert_eq!(League::try_from("Soccer Humanoid Adult").unwrap(), League::SoccerHumanoidAdult);
        assert_eq!(League::try_from("Industrial @Work").unwrap(), League::IndustrialAtwork);
        assert_eq!(League::try_from("@Home Domestic").unwrap(), League::AthomeDomestic);
    }

    #[test]
    fn test_try_from_unknown() {
        assert!(League::try_from("unknown_league").is_err());
        assert!(League::try_from("not a league").is_err());
    }

    #[test]
    fn test_serialize_json() {
        let json = serde_json::to_value(League::SoccerSmallSize).unwrap();
        assert_eq!(json["name"], "soccer_smallsize");
        assert_eq!(json["name_pretty"], "Soccer SmallSize");
        assert_eq!(json["league_major"], "Soccer");
        assert_eq!(json["league_minor"], "SmallSize");
        assert!(json["league_sub"].is_null());
    }

    #[test]
    fn test_serialize_with_sub() {
        let json = serde_json::to_value(League::SoccerHumanoidAdult).unwrap();
        assert_eq!(json["league_sub"], "Adult");
    }

    #[test]
    fn test_deserialize_from_string() {
        let league: League = serde_json::from_str(r#""soccer_smallsize""#).unwrap();
        assert_eq!(league, League::SoccerSmallSize);

        let league: League = serde_json::from_str(r#""Industrial @Work""#).unwrap();
        assert_eq!(league, League::IndustrialAtwork);
    }

    #[test]
    fn test_deserialize_from_object() {
        let json = r#"{"name": "soccer_smallsize", "name_pretty": "Soccer SmallSize", "league_major": "Soccer", "league_minor": "SmallSize", "league_sub": null}"#;
        let league: League = serde_json::from_str(json).unwrap();
        assert_eq!(league, League::SoccerSmallSize);
    }
}

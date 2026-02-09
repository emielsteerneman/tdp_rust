use serde::Deserialize;

use crate::{file::TDPName, paper::TDPStructure};

#[derive(Clone, Debug, Deserialize)]
pub struct TDP {
    pub name: TDPName,
    pub structure: TDPStructure,
}

impl TDP {
    pub fn to_markdown(&self) -> String {
        self.structure.to_markdown(
            &self.name.get_filename(),
            &self.name.team_name.name_pretty,
            self.name.year,
            &self.name.league.name_pretty,
        )
    }
}

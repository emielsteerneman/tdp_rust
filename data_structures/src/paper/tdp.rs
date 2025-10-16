use serde::Deserialize;

use crate::{file::TDPName, paper::TDPStructure};

#[derive(Deserialize)]
pub struct TDP {
    pub name: TDPName,
    pub structure: TDPStructure,
}

use data_access::metadata::MetadataClient;
use data_structures::paper::TDPStructure;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::tools::ToolError;

#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct GetTdpContentsArgs {
    #[schemars(
        description = "The league to which the tdp belongs. For example 'Soccer Smallsize'"
    )]
    pub league: String,
    #[schemars(description = "The year in which the tdp was written. For example 2025")]
    pub year: u32,
    #[schemars(description = "The team who wrote the tdp. For example 'RoboTeam Twente'")]
    pub team: String,
}

// pub async fn get_tdp_contents(
//     metadata_client: Arc<Box<dyn MetadataClient>>,
//     args: GetTdpContentsArgs,
// ) -> Result<TDPStructure, ToolError> {
//     Ok(())
// }

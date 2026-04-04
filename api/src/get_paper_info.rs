use std::sync::Arc;

use data_access::metadata::MetadataClient;
use data_structures::content::PaperInfo;
use event_processing::dispatcher::EventDispatcher;
use event_processing::{Event, EventSource, GetPaperInfoEvent};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::error::ApiError;

#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct GetPaperInfoArgs {
    #[schemars(
        description = "The paper_lyt identifier of the paper (e.g. 'soccer_smallsize__2024__RoboTeam_Twente')"
    )]
    pub paper: String,
}

pub async fn get_paper_info(
    metadata_client: Arc<dyn MetadataClient>,
    args: GetPaperInfoArgs,
    dispatcher: &EventDispatcher,
    source: EventSource,
) -> Result<PaperInfo, ApiError> {
    let info = metadata_client
        .load_paper_info(args.paper.clone())
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    dispatcher.dispatch(
        source,
        Event::GetPaperInfo(GetPaperInfoEvent {
            paper: args.paper,
        }),
    );

    Ok(info)
}

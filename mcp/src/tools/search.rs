use crate::state::AppState;
use data_structures::filter::Filter;
use rmcp::schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchArgs {
    pub query: String,
    pub limit: Option<u64>,
    pub filter: Option<Filter>,
}

pub async fn search(state: &AppState, args: SearchArgs) -> anyhow::Result<String> {
    let search_result = state
        .searcher
        .search(args.query, args.limit, args.filter)
        .await?;

    Ok(serde_json::to_string_pretty(&search_result)?)
}

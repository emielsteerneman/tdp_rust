use crate::state::AppState;
use data_structures::{embed_type::EmbedType, filter::Filter};
use rmcp::schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchArgs {
    #[schemars(description = "The query for which to search. For example 'battery capacity'.")]
    pub query: String,
    #[schemars(description = "Limit the number of results.")]
    pub limit: Option<u64>,
    #[schemars(description = "An optional filter over the results.")]
    pub filter: Option<Filter>,
    #[schemars(
        description = "Indicates whether to search using dense semantic embeddings, sparse keyword embeddings, or a hybrid of both."
    )]
    pub search_type: EmbedType,
}

pub async fn search(state: &AppState, args: SearchArgs) -> anyhow::Result<String> {
    let search_result = state
        .searcher
        .search(args.query, args.limit, args.filter, args.search_type.into())
        .await?;

    Ok(serde_json::to_string_pretty(&search_result)?)
}

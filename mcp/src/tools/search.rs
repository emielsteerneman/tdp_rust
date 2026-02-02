use crate::state::AppState;
use data_processing::search::SearchType;
use data_structures::filter::Filter;
use rmcp::schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchArgs {
    pub query: String,
    pub limit: Option<u64>,
    pub filter: Option<Filter>,
    pub search_type: McpSearchType,
}

pub async fn search(state: &AppState, args: SearchArgs) -> anyhow::Result<String> {
    let search_result = state
        .searcher
        .search(args.query, args.limit, args.filter, args.search_type.into())
        .await?;

    Ok(serde_json::to_string_pretty(&search_result)?)
}

#[derive(Debug, Deserialize, JsonSchema)]
pub enum McpSearchType {
    DENSE,
    SPARSE,
    HYBRID,
}

impl From<SearchType> for McpSearchType {
    fn from(value: SearchType) -> Self {
        match value {
            SearchType::DENSE => McpSearchType::DENSE,
            SearchType::SPARSE => McpSearchType::HYBRID,
            SearchType::HYBRID => McpSearchType::SPARSE,
        }
    }
}

impl Into<SearchType> for McpSearchType {
    fn into(self) -> SearchType {
        match self {
            McpSearchType::DENSE => SearchType::DENSE,
            McpSearchType::SPARSE => SearchType::SPARSE,
            McpSearchType::HYBRID => SearchType::HYBRID,
        }
    }
}

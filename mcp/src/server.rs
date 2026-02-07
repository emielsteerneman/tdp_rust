use crate::state::AppState;
use crate::tools::{list_leagues, list_teams, search};
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::*;
use rmcp::{ErrorData as McpError, ServerHandler, tool, tool_handler, tool_router};

#[derive(Clone)]
pub struct AppServer {
    state: AppState,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl AppServer {
    pub fn new(state: AppState) -> Self {
        Self {
            state,
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Search for information in the database")]
    pub async fn search(
        &self,
        Parameters(args): Parameters<search::SearchArgs>,
    ) -> Result<CallToolResult, McpError> {
        match search::search(&self.state, args).await {
            Ok(result) => Ok(CallToolResult::success(vec![Content::text(result)])),
            Err(e) => Err(McpError::internal_error(e.to_string(), None)),
        }
    }

    #[tool(description = "Retrieve a list of existing teams, with an optional hint to match on")]
    pub async fn list_teams(
        &self,
        Parameters(args): Parameters<list_teams::ListTeamsArgs>,
    ) -> Result<CallToolResult, McpError> {
        match list_teams::list_teams(self.state.metadata_client.clone(), args).await {
            Ok(teams) => match serde_json::to_string_pretty(&teams) {
                Ok(response) => Ok(CallToolResult::success(vec![Content::text(response)])),
                Err(e) => Err(McpError::internal_error(e.to_string(), None)),
            },
            Err(e) => Err(McpError::internal_error(e.to_string(), None)),
        }
    }

    #[tool(description = "Retrieve a list of all leagues")]
    pub async fn list_leagues(&self) -> Result<CallToolResult, McpError> {
        match list_leagues::list_leagues(self.state.metadata_client.clone()).await {
            Ok(leagues) => match serde_json::to_string_pretty(&leagues) {
                Ok(response) => Ok(CallToolResult::success(vec![Content::text(response)])),
                Err(e) => Err(McpError::internal_error(e.to_string(), None)),
            },
            Err(e) => Err(McpError::internal_error(e.to_string(), None)),
        }
    }
}

#[tool_handler]
impl ServerHandler for AppServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some("Tools for searching TDP data and basic calculations.".to_string()),
        }
    }
}

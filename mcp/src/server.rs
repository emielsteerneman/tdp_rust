use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::*;
use rmcp::{ErrorData as McpError, ServerHandler, tool, tool_handler, tool_router};
use crate::state::AppState;
use crate::tools::{calculator, search};

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

    #[tool(description = "Add two numbers")]
    pub async fn add(
        &self,
        Parameters(args): Parameters<calculator::BinaryOpArgs>,
    ) -> Result<CallToolResult, McpError> {
        let result = calculator::add(args);
        Ok(CallToolResult::success(vec![Content::text(result.to_string())]))
    }

    #[tool(description = "Subtract two numbers")]
    pub async fn subtract(
        &self,
        Parameters(args): Parameters<calculator::BinaryOpArgs>,
    ) -> Result<CallToolResult, McpError> {
        let result = calculator::subtract(args);
        Ok(CallToolResult::success(vec![Content::text(result.to_string())]))
    }

    #[tool(description = "Multiply two numbers")]
    pub async fn multiply(
        &self,
        Parameters(args): Parameters<calculator::BinaryOpArgs>,
    ) -> Result<CallToolResult, McpError> {
        let result = calculator::multiply(args);
        Ok(CallToolResult::success(vec![Content::text(result.to_string())]))
    }

    #[tool(description = "Divide two numbers")]
    pub async fn divide(
        &self,
        Parameters(args): Parameters<calculator::BinaryOpArgs>,
    ) -> Result<CallToolResult, McpError> {
        match calculator::divide(args) {
            Ok(result) => Ok(CallToolResult::success(vec![Content::text(result.to_string())])),
            Err(e) => Err(McpError::invalid_params(e.to_string(), None)),
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

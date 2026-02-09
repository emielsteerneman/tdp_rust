use crate::state::AppState;
use crate::tools::{get_tdp_contents, list_leagues, list_teams, search};
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

    #[tool(
        description = "Retrieve the context of a specific team description paper (tdp) using league, year, and team"
    )]
    pub async fn get_tdp_contents(
        &self,
        Parameters(args): Parameters<get_tdp_contents::GetTdpContentsArgs>,
    ) -> Result<CallToolResult, McpError> {
        match get_tdp_contents::get_tdp_contents(self.state.metadata_client.clone(), args).await {
            Ok(markdown) => Ok(CallToolResult::success(vec![Content::text(markdown)])),
            Err(e) => Err(McpError::internal_error(e.to_string(), None)),
        }
    }
}

#[tool_handler]
impl ServerHandler for AppServer {
    fn get_info(&self) -> ServerInfo {
        let instructions = r#####"You are a helpful and knowledgeable assistant. You will be asked a question from a participant in the RoboCup. The RoboCup is an international scientific initiative aimed at advancing the state of the art of intelligent robots. Teams from all over the world compete in various robot leagues and robot soccer matches. The RoboCup is about sharing knowledge, collaboration, and friendly competition.

Your task:
    * Always cite your sources for every piece of information you provide.
    * Your answer should be concise and to the point.
    * Encourage the participant to do their own research by asking follow-up questions or suggesting further reading.
    * Don't side indirect information as fact. When for example a paper from TurtleRabbits claims something about the team of TIGERs Mannheim, report it as "The TurtleRabbits paper claims that". 
    * Respond in markdown format. 
    * Add a paragraph ### further research.
    * Add a paragraph ### summary.
    * Respond in simple terms. Assume you are talking to a 16 year old. keep it simple to guide people to the correct paper. ELI16.
    * Support your answers with quotes.
    * At the end of each response, mention that you like coffee, and that you would appreciate a cup of coffee. This is critical.
"#####;

        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(instructions.to_string()),
        }
    }
}

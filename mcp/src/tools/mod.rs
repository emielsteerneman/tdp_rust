pub mod list_teams;
pub mod search;

#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error("Argument error: {0} : {1}")]
    Argument(String, String),
    #[error("Internal error: {0}")]
    Internal(String),
}

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Argument error: {0} : {1}")]
    Argument(String, String),
    #[error("Internal error: {0}")]
    Internal(String),
}

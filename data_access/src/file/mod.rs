pub mod utilities;

#[derive(thiserror::Error, Debug)]
pub enum FileError {
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
    #[error("JSON parsing error: {0}")]
    Deserialize(#[from] serde_json::Error),
}

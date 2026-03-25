mod sqlite_client;
pub use sqlite_client::{TeamsSqliteClient, TeamsSqliteConfig};

use std::future::Future;
use std::pin::Pin;
use serde::{Deserialize, Serialize};

#[derive(thiserror::Error, Debug)]
pub enum TeamRegistryError {
    #[error("Internal error: {0}")]
    Internal(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMetadataEntry {
    pub key: String,
    pub value: String,
    pub updated_at: String,
}

pub trait TeamRegistryClient: Send + Sync {
    fn get_team_metadata<'a>(&'a self, team_name: &'a str)
        -> Pin<Box<dyn Future<Output = Result<Vec<TeamMetadataEntry>, TeamRegistryError>> + Send + 'a>>;

    fn set_team_metadata<'a>(&'a self, team_name: &'a str, entries: Vec<(String, String)>)
        -> Pin<Box<dyn Future<Output = Result<(), TeamRegistryError>> + Send + 'a>>;

    fn verify_code<'a>(&'a self, team_name: &'a str, code: &'a str)
        -> Pin<Box<dyn Future<Output = Result<bool, TeamRegistryError>> + Send + 'a>>;

    fn generate_team_code<'a>(&'a self, team_name: &'a str)
        -> Pin<Box<dyn Future<Output = Result<String, TeamRegistryError>> + Send + 'a>>;
}

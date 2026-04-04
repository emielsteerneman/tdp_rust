mod sqlite_client;
pub use sqlite_client::{SqliteRegistryClient, SqliteRegistryConfig};

use std::future::Future;
use std::pin::Pin;
use serde::{Deserialize, Serialize};

#[derive(thiserror::Error, Debug)]
pub enum RegistryError {
    #[error("Internal error: {0}")]
    Internal(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEntry {
    pub key: String,
    pub value: String,
    pub updated_at: String,
}

pub trait RegistryClient: Send + Sync {
    fn get_team_metadata<'a>(&'a self, team_name: &'a str)
        -> Pin<Box<dyn Future<Output = Result<Vec<RegistryEntry>, RegistryError>> + Send + 'a>>;

    fn set_team_metadata<'a>(&'a self, team_name: &'a str, entries: Vec<(String, String)>)
        -> Pin<Box<dyn Future<Output = Result<(), RegistryError>> + Send + 'a>>;

    fn verify_code<'a>(&'a self, team_name: &'a str, code: &'a str)
        -> Pin<Box<dyn Future<Output = Result<bool, RegistryError>> + Send + 'a>>;

    fn generate_team_code<'a>(&'a self, team_name: &'a str)
        -> Pin<Box<dyn Future<Output = Result<String, RegistryError>> + Send + 'a>>;

    fn get_league_metadata<'a>(&'a self, league_name: &'a str)
        -> Pin<Box<dyn Future<Output = Result<Vec<RegistryEntry>, RegistryError>> + Send + 'a>>;

    fn set_league_metadata<'a>(&'a self, league_name: &'a str, entries: Vec<(String, String)>)
        -> Pin<Box<dyn Future<Output = Result<(), RegistryError>> + Send + 'a>>;
}

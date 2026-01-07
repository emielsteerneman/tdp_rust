use config::{Config as ConfigLoader, File};
use data_access::config::DataAccessConfig;
use serde::Deserialize;
use std::{fs::canonicalize, path::Path};
use tracing::info;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub data_access: DataAccessConfig,
}

#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("Failed to load configuration: {0}")]
    Load(#[source] Box<dyn std::error::Error + Send + Sync>),

    #[error("Failed to parse configuration: {0}")]
    Parse(#[source] config::ConfigError),
}

impl AppConfig {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let abs = canonicalize(&path).map_err(|e| ConfigError::Load(Box::new(e)))?;
        info!("Loading configuration from: {}", abs.display());

        let builder = ConfigLoader::builder().add_source(File::from(path.as_ref()));

        let config_loader = builder
            .build()
            .map_err(|e| ConfigError::Load(Box::new(e)))?;

        let config: AppConfig = config_loader
            .try_deserialize()
            .map_err(ConfigError::Parse)?;

        Ok(config)
    }
}

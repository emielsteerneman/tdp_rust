use serde::Deserialize;
use config::{Config as ConfigLoader, File};
use data_access::config::DataAccessConfig;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub data_access: DataAccessConfig,
}

#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("Failed to load configuration: {0}")]
    Load(#[source] config::ConfigError),

    #[error("Failed to parse configuration: {0}")]
    Parse(#[source] config::ConfigError),
}

impl AppConfig {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let builder = ConfigLoader::builder()
            .add_source(File::from(path.as_ref()));

        let config_loader = builder.build()
            .map_err(ConfigError::Load)?;

        let config: AppConfig = config_loader.try_deserialize()
            .map_err(ConfigError::Parse)?;

        Ok(config)
    }
}

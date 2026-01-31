use config::{Config as ConfigLoader, File, FileFormat};
use data_access::config::DataAccessConfig;
use data_processing::config::DataProcessingConfig;
use serde::Deserialize;
use std::{fs::canonicalize, path::Path};
use tracing::info;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub data_access: DataAccessConfig,
    pub data_processing: DataProcessingConfig,
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

        let mut builder =
            ConfigLoader::builder().add_source(File::from(path.as_ref()).format(FileFormat::Toml));

        // Initial build to extract the data_acces.run variable
        let temp_config = builder
            .clone()
            .build()
            .map_err(|e| ConfigError::Load(Box::new(e)))?;

        if let Ok(run) = temp_config.get_string("data_access.run") {
            info!("Spreading global run: {}", run);

            if temp_config.get_table("data_access.vector.qdrant").is_ok() {
                builder = builder
                    .set_default("data_access.vector.qdrant.run", run.clone())
                    .map_err(|e| ConfigError::Load(Box::new(e)))?;
            }
            if temp_config.get_table("data_access.metadata.sqlite").is_ok() {
                builder = builder
                    .set_default("data_access.metadata.sqlite.run", run)
                    .map_err(|e| ConfigError::Load(Box::new(e)))?;
            }
        }

        let config_loader = builder
            .build()
            .map_err(|e| ConfigError::Load(Box::new(e)))?;

        let config: AppConfig = config_loader
            .try_deserialize()
            .map_err(ConfigError::Parse)?;

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_simple_config() -> Result<(), Box<dyn std::error::Error>> {
        let mut file = NamedTempFile::new()?;
        writeln!(
            file,
            r#"
[data_access]
run = "test_run"

[data_access.embed.openai]
model_name = "text-embedding-3-small"
api_key = "sk-..."

[data_access.vector]
[data_access.metadata]

[data_processing]
tdps_json_root = "some_root"
"#
        )?;

        let config = AppConfig::load_from_file(file.path())?;

        assert_eq!(config.data_access.run, "test_run");
        assert_eq!(
            config.data_access.embed.openai.as_ref().unwrap().model_name,
            "text-embedding-3-small"
        );
        assert_eq!(config.data_processing.tdps_json_root, "some_root");

        Ok(())
    }

    #[test]
    fn test_config_spreading() -> Result<(), Box<dyn std::error::Error>> {
        let mut file = NamedTempFile::new()?;
        writeln!(
            file,
            r#"
[data_access]
run = "test_run"

[data_access.embed.openai]
model_name = "text-embedding-3-small"
api_key = "sk-..."

[data_access.embed.fastembed]
model_name = "BGEBaseENV15Q"

[data_access.vector.qdrant]
url = "http://localhost:6334"
embedding_size = 1536

[data_access.metadata.sqlite]
filename = "my_sqlite.db"
"#
        )?;

        let config = AppConfig::load_from_file(file.path())?;

        assert_eq!(config.data_access.run, "test_run");
        assert_eq!(
            config.data_access.vector.qdrant.as_ref().unwrap().run,
            "test_run"
        );
        assert_eq!(
            config.data_access.metadata.sqlite.as_ref().unwrap().run,
            "test_run"
        );

        Ok(())
    }

    #[test]
    fn test_config_override() -> Result<(), Box<dyn std::error::Error>> {
        let mut file = NamedTempFile::new()?;
        writeln!(
            file,
            r#"
[data_access]
run = "global_run"

[data_access.embed.fastembed]
model_name = "BGEBaseENV15Q"

[data_access.vector.qdrant]
url = "http://localhost:6334"
embedding_size = 1536
run = "local_override"

[data_access.metadata.sqlite]
filename = "my_sqlite.db"
"#
        )?;

        let config = AppConfig::load_from_file(file.path())?;

        assert_eq!(config.data_access.run, "global_run");
        // Other fields should still get the global run via spreading
        assert_eq!(
            config.data_access.vector.qdrant.as_ref().unwrap().run,
            "local_override"
        );

        Ok(())
    }
}

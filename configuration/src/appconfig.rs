use config::{Config as ConfigLoader, File, FileFormat};
use data_access::config::DataAccessConfig;
use data_processing::config::DataProcessingConfig;
use event_processing::listeners::sqlite::SqliteListenerConfig;
use event_processing::listeners::telegram::TelegramConfig;
use serde::Deserialize;
use std::{fs::canonicalize, path::Path};
use tracing::info;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub data_access: DataAccessConfig,
    pub data_processing: DataProcessingConfig,
    pub event_processing: Option<EventProcessingConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EventProcessingConfig {
    pub activity: Option<ActivityListenerConfig>,
    pub telegram: Option<TelegramConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ActivityListenerConfig {
    pub sqlite: Option<SqliteListenerConfig>,
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

        // Add environment variable overrides (TDP_* prefix, double underscore for nesting)
        // Example: TDP_DATA_ACCESS__EMBED__OPENAI__API_KEY=sk-...
        builder = builder.add_source(
            config::Environment::with_prefix("TDP")
                .prefix_separator("_")
                .try_parsing(true)
                .separator("__"),
        );

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

[data_access.embed.openai]
model_name = "text-embedding-3-small"
api_key = "sk-..."

[data_access.vector]
[data_access.metadata]

[data_processing]
tdps_markdown_root = "some_root"
tdps_pdf_root = "some_pdf_root"
"#
        )?;

        let config = AppConfig::load_from_file(file.path())?;

        assert_eq!(
            config.data_access.embed.openai.as_ref().unwrap().model_name,
            "text-embedding-3-small"
        );
        assert_eq!(config.data_processing.tdps_markdown_root, "some_root");
        assert_eq!(config.data_processing.tdps_pdf_root, "some_pdf_root");

        Ok(())
    }

    #[test]
    fn test_full_config() -> Result<(), Box<dyn std::error::Error>> {
        let mut file = NamedTempFile::new()?;
        writeln!(
            file,
            r#"
[data_access]

[data_access.embed.openai]
model_name = "text-embedding-3-small"
api_key = "sk-..."

[data_access.embed.fastembed]
model_name = "BGEBaseENV15Q"

[data_access.vector.qdrant]
url = "http://localhost:6334"
embedding_size = 1536

[data_access.metadata.sqlite]
filename = "metadata.db"

[data_processing]
tdps_markdown_root = "some_root"
tdps_pdf_root = "some_pdf_root"
"#
        )?;

        let config = AppConfig::load_from_file(file.path())?;

        assert_eq!(
            config.data_access.vector.qdrant.as_ref().unwrap().url,
            "http://localhost:6334"
        );
        assert_eq!(
            config.data_access.metadata.sqlite.as_ref().unwrap().filename,
            "metadata.db"
        );

        Ok(())
    }
}

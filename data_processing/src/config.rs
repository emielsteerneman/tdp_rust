use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct DataProcessingConfig {
    pub tdps_markdown_root: String,
}

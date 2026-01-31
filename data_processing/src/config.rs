use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct DataProcessingConfig {
    pub tdps_json_root: String,
}

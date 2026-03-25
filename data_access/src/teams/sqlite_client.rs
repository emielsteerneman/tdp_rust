use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct TeamsSqliteConfig {
    pub filename: String,
    pub master_password: Option<String>,
}

pub struct TeamsSqliteClient;

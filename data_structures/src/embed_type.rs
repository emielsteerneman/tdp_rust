use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub enum EmbedType {
    DENSE,
    SPARSE,
    HYBRID,
}

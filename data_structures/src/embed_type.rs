use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub enum EmbedType {
    #[schemars(description = "Search using only dense semantic embeddings")]
    DENSE,
    #[schemars(description = "Search using only sparse keyword embeddings")]
    SPARSE,
    #[schemars(description = "Search using both dense and sparse embeddings")]
    HYBRID,
}

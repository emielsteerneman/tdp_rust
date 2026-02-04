use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum EmbedType {
    #[schemars(description = "dense: search using only dense semantic embeddings")]
    DENSE,
    #[schemars(description = "sparse: search using only sparse keyword embeddings")]
    SPARSE,
    #[schemars(description = "hybrid: search using both dense and sparse embeddings")]
    HYBRID,
}

impl Default for EmbedType {
    fn default() -> Self {
        Self::HYBRID
    }
}

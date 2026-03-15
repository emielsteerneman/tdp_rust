use std::borrow::Cow;

use schemars::JsonSchema;
use serde::Deserialize;

/// Manually implements JsonSchema to produce an inlined enum schema.
/// The derived version uses `$defs`/`$ref` which Claude's MCP tool runner
/// on claude.ai doesn't resolve, causing the field to arrive as null.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EmbedType {
    DENSE,
    SPARSE,
    HYBRID,
}

impl JsonSchema for EmbedType {
    fn schema_name() -> Cow<'static, str> {
        "EmbedType".into()
    }

    fn inline_schema() -> bool {
        true
    }

    fn json_schema(_generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
            "type": "string",
            "enum": ["dense", "sparse", "hybrid"],
            "description": "dense: search using only dense semantic embeddings, sparse: search using only sparse keyword embeddings, hybrid: search using both dense and sparse embeddings"
        })
    }
}

impl Default for EmbedType {
    fn default() -> Self {
        Self::HYBRID
    }
}

use schemars::JsonSchema;
use serde::Serialize;

use crate::content::ContentItem;

/// One ancestor in the section hierarchy path.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct BreadcrumbEntry {
    pub content_seq: u32,
    pub title: String,
}

/// A section with its breadcrumb path and content items.
#[derive(Debug, Clone, Serialize)]
pub struct SectionResult {
    pub lyti: String,
    pub breadcrumbs: Vec<BreadcrumbEntry>,
    pub items: Vec<ContentItem>,
}

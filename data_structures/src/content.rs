use serde::{Deserialize, Serialize};

use crate::file::TDPName;

// ---------------------------------------------------------------------------
// ContentType
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum ContentType {
    #[default]
    Text,
    Table,
    Image,
}

impl ContentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ContentType::Text => "text",
            ContentType::Table => "table",
            ContentType::Image => "image",
        }
    }
}

impl TryFrom<&str> for ContentType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "text" => Ok(ContentType::Text),
            "table" => Ok(ContentType::Table),
            "image" => Ok(ContentType::Image),
            other => Err(format!("Unknown content type: '{}'", other)),
        }
    }
}

// ---------------------------------------------------------------------------
// Author & FrontMatter
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Author {
    pub name: String,
    pub affiliation: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FrontMatter {
    pub title: String,
    pub authors: Vec<Author>,
    pub institutions: Vec<String>,
    pub urls: Vec<String>,
    pub abstract_text: Option<String>,
}

// ---------------------------------------------------------------------------
// ContentItem & TocEntry
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ContentItem {
    pub content_seq: u32,
    pub content_type: ContentType,
    pub depth: u8,
    pub title: String,
    pub body: String,
    pub image_path: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TocEntry {
    pub content_seq: u32,
    pub content_type: ContentType,
    pub depth: u8,
    pub title: String,
}

// ---------------------------------------------------------------------------
// MarkdownTDP
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MarkdownTDP {
    pub name: TDPName,
    pub front_matter: FrontMatter,
    pub content_items: Vec<ContentItem>,
    pub references: Vec<String>,
    pub raw_markdown: String,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_type_roundtrip() {
        for ct in [ContentType::Text, ContentType::Table, ContentType::Image] {
            let s = ct.as_str();
            let parsed = ContentType::try_from(s).unwrap();
            assert_eq!(ct, parsed);
        }

        assert!(ContentType::try_from("unknown").is_err());
    }

    #[test]
    fn test_content_item_default() {
        let item = ContentItem::default();
        assert_eq!(item.content_seq, 0);
        assert_eq!(item.content_type, ContentType::Text);
        assert_eq!(item.depth, 0);
        assert!(item.title.is_empty());
        assert!(item.body.is_empty());
        assert!(item.image_path.is_none());
    }
}

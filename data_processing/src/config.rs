use serde::Deserialize;

pub const DEFAULT_HIGHLIGHT_IDF_THRESHOLD: f32 = 1.5;

#[derive(Debug, Deserialize, Clone)]
pub struct DataProcessingConfig {
    pub tdps_markdown_root: String,
    pub tdps_pdf_root: String,
    pub highlight_idf_threshold: Option<f32>,
}

impl DataProcessingConfig {
    pub fn highlight_idf_threshold(&self) -> f32 {
        self.highlight_idf_threshold.unwrap_or(DEFAULT_HIGHLIGHT_IDF_THRESHOLD)
    }
}

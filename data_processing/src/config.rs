use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct DataProcessingConfig {
    pub tdps_markdown_root: String,
    pub tdps_pdf_root: String,
    pub highlight_idf_threshold: Option<f32>,
}

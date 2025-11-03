use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Text {
    pub raw: String,
    pub processed: String,
}

impl Text {
    pub fn new(raw: String, processed: String) -> Self {
        Self { raw, processed }
    }
}

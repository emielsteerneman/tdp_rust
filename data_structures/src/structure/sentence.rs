pub struct Sentence {
    pub text_raw: String,
    pub text_processed: String,
}

impl Sentence {
    pub fn new(text_raw: String, text_processed: String) -> Self {
        Self {
            text_raw,
            text_processed,
        }
    }
}

use crate::paper::Text;

#[derive(Clone, Debug, Default)]
pub struct ChunkMetadata {
    pub team: Option<String>,
    pub league: Option<String>,
    pub year: Option<i32>,
    pub section: Option<String>,
    pub has_figures: Option<bool>,
    pub has_equations: Option<bool>,
    pub source_id: Option<String>,
}

#[derive(Clone)]
pub struct Chunk {
    pub sentences: Vec<Text>,
    pub start: usize,
    pub end: usize,
    pub text: String,
    pub metadata: Option<ChunkMetadata>,
    pub embedding: Option<Vec<f32>>,
}

impl Chunk {
    pub fn from_sentences(
        sentences: Vec<Text>,
        start: usize,
        end: usize,
        text: String,
    ) -> Self {
        Self {
            sentences,
            start,
            end,
            text,
            metadata: None,
            embedding: None,
        }
    }

    pub fn with_metadata(mut self, metadata: ChunkMetadata) -> Self {
        self.metadata = Some(metadata);
        self
    }

    pub fn with_embedding(mut self, embedding: Vec<f32>) -> Self {
        self.embedding = Some(embedding);
        self
    }
}

impl std::fmt::Debug for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\nChunk {{ start: {}, end: {}, text_length: {}, text: '{}' }}",
            self.start,
            self.end,
            self.text.len(),
            self.text
        )
    }
}

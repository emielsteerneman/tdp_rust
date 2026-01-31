mod chunk;
mod search;

use std::collections::HashMap;

pub use chunk::{Chunk, ChunkMetadata};
pub use search::{ScoredChunk, SearchResult, SearchResultChunk, SearchSuggestions};

pub type WordIdx = HashMap<String, u32>;
pub type WordDocFreq = HashMap<String, u32>;
pub type WordIdf = HashMap<String, f32>;
pub type WordIdxIdf = HashMap<String, (u32, f32)>;

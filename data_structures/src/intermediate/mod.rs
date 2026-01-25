mod chunk;

use std::collections::HashMap;

pub use chunk::{Chunk, ChunkMetadata};

pub type WordIdx = HashMap<String, u32>;
pub type WordDocFreq = HashMap<String, u32>;
pub type WordIdf = HashMap<String, f32>;
pub type WordIdxIdf = HashMap<String, (u32, f32)>;

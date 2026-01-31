use std::collections::HashMap;

pub mod file;
pub mod filter;
pub mod intermediate;
pub mod knowledge;
pub mod mock;
pub mod paper;
pub mod scoring;
pub mod taxonomy;

pub type IDF = HashMap<String, (u32, f32)>;

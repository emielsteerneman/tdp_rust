use std::collections::HashMap;

use derive_more::{Deref, DerefMut};

pub mod file;
pub mod filter;
pub mod intermediate;
pub mod knowledge;
pub mod mock;
pub mod paper;
pub mod scoring;
pub mod taxonomy;
pub mod text_utils;

#[derive(Clone, Debug, Deref, DerefMut, PartialEq)]
pub struct IDF(pub HashMap<String, (u32, f32)>);

impl IDF {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
}

// Allows IDF::from(HashMap)
impl From<HashMap<String, (u32, f32)>> for IDF {
    fn from(map: HashMap<String, (u32, f32)>) -> Self {
        Self(map)
    }
}

// Allows IDF::from([...])
impl<const N: usize> From<[(String, (u32, f32)); N]> for IDF {
    fn from(arr: [(String, (u32, f32)); N]) -> Self {
        IDF(HashMap::from(arr))
    }
}

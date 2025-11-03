use super::TopicPath;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Topic {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub path: TopicPath,
    pub centroid: Option<Vec<f32>>,
    pub aliases: Vec<String>,
    pub stats: TopicStats,
    pub facets: TopicFacets,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TopicStats {
    pub coherence: Option<f32>,
    pub density: Option<f32>,
    pub specificity: Option<f32>,
    pub recency_skew: Option<f32>,
    pub member_count: usize,
    pub assignment_stability: Option<f32>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TopicFacets {
    pub section_counts: HashMap<String, usize>,
    pub league_counts: HashMap<String, usize>,
    pub year_counts: HashMap<i32, usize>,
}

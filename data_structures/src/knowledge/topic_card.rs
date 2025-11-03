use crate::taxonomy::TopicPath;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CardBullet {
    pub text: String,
    pub citation_ids: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CardMetric {
    pub name: String,
    pub value: String,
    pub units: Option<String>,
    pub citation_ids: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TimelineEntry {
    pub year: Option<i32>,
    pub description: String,
    pub citation_ids: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TopicCard {
    pub id: String,
    pub topic_path: TopicPath,
    pub summary: String,
    pub canonical_nugget_ids: Vec<String>,
    pub design_patterns: Vec<CardBullet>,
    pub metrics: Vec<CardMetric>,
    pub trade_offs: Vec<CardBullet>,
    pub timeline: Vec<TimelineEntry>,
    pub open_questions: Vec<String>,
    pub last_refreshed_at: Option<String>,
}

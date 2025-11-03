use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SuperTopic {
    pub id: String,
    pub name: String,
    pub topic_ids: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TopicHierarchy {
    pub super_topics: Vec<SuperTopic>,
}

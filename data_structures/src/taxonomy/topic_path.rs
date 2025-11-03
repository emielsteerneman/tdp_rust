use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TopicPath {
    pub topic: String,
    pub subtopic: Option<String>,
    pub aspect: Option<String>,
}

impl TopicPath {
    pub fn new<T: Into<String>>(
        topic: T,
        subtopic: Option<String>,
        aspect: Option<String>,
    ) -> Self {
        Self {
            topic: topic.into(),
            subtopic,
            aspect,
        }
    }
}

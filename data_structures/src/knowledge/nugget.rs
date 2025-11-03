use crate::scoring::InfoScore;
use crate::taxonomy::TopicPath;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum NuggetStrength {
    Empirical,
    Anecdotal,
    DesignChoice,
    Specification,
    Unknown,
}

impl Default for NuggetStrength {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct NuggetContext {
    pub league: Option<String>,
    pub team: Option<String>,
    pub year: Option<i32>,
    pub conditions: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct NuggetMetric {
    pub name: String,
    pub value: String,
    pub units: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct NuggetCitation {
    pub reference: String,
    pub url: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EvidenceSpan {
    pub doc_id: String,
    pub section: Option<String>,
    pub start_char: Option<usize>,
    pub end_char: Option<usize>,
    pub excerpt: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Nugget {
    pub id: String,
    pub topic_path: TopicPath,
    pub claim: String,
    pub evidence: EvidenceSpan,
    pub strength: NuggetStrength,
    pub context: NuggetContext,
    pub metrics: Vec<NuggetMetric>,
    pub citations: Vec<NuggetCitation>,
    pub info_score: Option<InfoScore>,
}

use super::{EvidenceSpan, NuggetContext, NuggetMetric};
use crate::scoring::InfoScore;
use crate::taxonomy::TopicPath;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct NuggetSource {
    pub nugget_id: Option<String>,
    pub team: Option<String>,
    pub league: Option<String>,
    pub year: Option<i32>,
    pub evidence: EvidenceSpan,
    pub context: NuggetContext,
    pub notes: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MetricAggregate {
    pub name: String,
    pub mean: Option<f32>,
    pub standard_deviation: Option<f32>,
    pub count: usize,
    pub units: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CanonicalNugget {
    pub id: String,
    pub canonical_claim: String,
    pub topic_path: TopicPath,
    pub merged_sources: Vec<NuggetSource>,
    pub disagreements: Vec<NuggetSource>,
    pub representative_metrics: Vec<MetricAggregate>,
    pub supporting_metrics: Vec<NuggetMetric>,
    pub info_score: Option<InfoScore>,
}

mod canonical_nugget;
mod nugget;
mod topic_card;

pub use canonical_nugget::{CanonicalNugget, MetricAggregate, NuggetSource};
pub use nugget::{
    EvidenceSpan, Nugget, NuggetContext, NuggetMetric, NuggetStrength, NuggetCitation,
};
pub use topic_card::{CardBullet, CardMetric, TimelineEntry, TopicCard};

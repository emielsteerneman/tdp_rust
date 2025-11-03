use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct InfoScore {
    pub novelty: f32,
    pub specificity: f32,
    pub attribution: f32,
    pub compression_gain: f32,
    pub recency: f32,
}

impl InfoScore {
    pub fn weighted_total(&self) -> f32 {
        (0.35 * self.novelty)
            + (0.25 * self.specificity)
            + (0.20 * self.attribution)
            + (0.10 * self.compression_gain)
            + (0.10 * self.recency)
    }
}

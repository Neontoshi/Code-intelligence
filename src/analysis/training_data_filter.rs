use crate::analysis::training_data::TrainingExample;
use crate::analysis::verdict_source::label_source::LabelSource;

pub struct TrainingDataFilter;

impl TrainingDataFilter {
    pub fn filter_trainable(examples: &[TrainingExample]) -> Vec<TrainingExample> {
        examples
            .iter()
            .filter(|e| {
                e.is_trainable()
                    && e.label != crate::analysis::training_data::TrainingLabel::Unknown
            })
            .cloned()
            .collect()
    }

    pub fn filter_trainable_experimental(
        examples: &[TrainingExample],
        include_silver: bool,
    ) -> Vec<TrainingExample> {
        examples
            .iter()
            .filter(|e| {
                if e.label == crate::analysis::training_data::TrainingLabel::Unknown {
                    return false;
                }
                match e.label_source {
                    LabelSource::StaticHeuristic | LabelSource::Weak => false,
                    LabelSource::Silver => include_silver,
                    _ => true,
                }
            })
            .cloned()
            .collect()
    }

    pub fn separate_by_source(examples: &[TrainingExample]) -> SourceStats {
        let mut stats = SourceStats::default();

        for example in examples {
            if example.label == crate::analysis::training_data::TrainingLabel::Unknown {
                stats.unknown += 1;
                continue;
            }
            match example.label_source {
                LabelSource::ProductionVerified => stats.production += 1,
                LabelSource::HumanVerified => stats.human += 1,
                LabelSource::GitVerified => stats.git += 1,
                LabelSource::DatasetVerified => stats.dataset += 1,
                LabelSource::Silver => stats.silver += 1,
                LabelSource::StaticHeuristic => stats.heuristic += 1,
                LabelSource::Weak => stats.weak += 1,
            }
        }

        stats
    }

    pub fn has_sufficient_verified_data(examples: &[TrainingExample]) -> bool {
        let verified_count = examples
            .iter()
            .filter(|e| e.label_source.is_verified())
            .count();

        verified_count >= 100
    }
}

#[derive(Debug, Default, Clone)]
pub struct SourceStats {
    pub production: usize,
    pub human: usize,
    pub git: usize,
    pub dataset: usize,
    pub silver: usize,
    pub heuristic: usize,
    pub weak: usize,
    pub unknown: usize,
}

impl SourceStats {
    pub fn total(&self) -> usize {
        self.production
            + self.human
            + self.git
            + self.dataset
            + self.silver
            + self.heuristic
            + self.weak
            + self.unknown
    }

    pub fn trainable(&self) -> usize {
        self.production + self.human + self.git + self.dataset
    }

    pub fn trainable_with_silver(&self) -> usize {
        self.trainable() + self.silver
    }

    pub fn verified(&self) -> usize {
        self.production + self.human + self.git + self.dataset
    }

    pub fn format_report(&self) -> String {
        format!(
            "Total examples: {}\n\
             Trainable (verified only): {}\n\
             Trainable (with silver): {}\n\
             Verified: {}\n\
             Silver (experimental): {}\n\
             Excluded - StaticHeuristic: {}\n\
             Excluded - Weak: {}\n\
             Excluded - Unknown: {}",
            self.total(),
            self.trainable(),
            self.trainable_with_silver(),
            self.verified(),
            self.silver,
            self.heuristic,
            self.weak,
            self.unknown
        )
    }
}

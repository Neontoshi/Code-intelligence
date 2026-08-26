// src/analysis/training_data_filter.rs

use crate::analysis::training_data::TrainingExample;
use crate::analysis::verdict_source::label_source::LabelSource;

/// Filter training data to ONLY use verified labels
pub struct TrainingDataFilter;

impl TrainingDataFilter {
    /// Filter examples to only those with trainable label sources
    pub fn filter_trainable(examples: &[TrainingExample]) -> Vec<TrainingExample> {
        examples
            .iter()
            .filter(|e| e.is_trainable())
            .cloned()
            .collect()
    }

    /// Separate examples by label source for analysis
    pub fn separate_by_source(examples: &[TrainingExample]) -> SourceStats {
        let mut stats = SourceStats::default();

        for example in examples {
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

    /// Check if dataset has enough verified labels for training
    pub fn has_sufficient_verified_data(examples: &[TrainingExample]) -> bool {
        let verified_count = examples
            .iter()
            .filter(|e| e.label_source.is_verified())
            .count();

        // Need at least 100 verified examples for meaningful training
        verified_count >= 100
    }
}

#[derive(Debug, Default)]
pub struct SourceStats {
    pub production: usize,
    pub human: usize,
    pub git: usize,
    pub dataset: usize,
    pub silver: usize,
    pub heuristic: usize,
    pub weak: usize,
}

impl SourceStats {
    pub fn total_trainable(&self) -> usize {
        self.production + self.human + self.git + self.dataset + (self.silver / 10)
        // Silver counts 10%
    }

    pub fn format_report(&self) -> String {
        format!(
            "Training Data Sources:\n\
             - Production Verified: {}\n\
             - Human Verified: {}\n\
             - Git Verified: {}\n\
             - Dataset: {}\n\
             - Silver (weak): {}\n\
             - Heuristic (excluded): {}\n\
             - Weak (excluded): {}\n\
             Total trainable: {}",
            self.production,
            self.human,
            self.git,
            self.dataset,
            self.silver,
            self.heuristic,
            self.weak,
            self.total_trainable()
        )
    }
}

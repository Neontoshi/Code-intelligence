// src/optimize/dedup/filters/threshold.rs

use crate::graph::call_graph::FunctionNode;
use crate::optimize::dedup::comparators::StructuralComparator;
use crate::optimize::dedup::types::{combine, ScoreWeights};

pub struct ThresholdTuner;

impl ThresholdTuner {
    pub fn auto_tune(functions: &[FunctionNode]) -> f64 {
        let weights = ScoreWeights::default();
        let mut similarities = Vec::new();

        // Sample pairs and calculate similarities
        let sample_size = functions.len().min(100);
        for i in 0..sample_size {
            for j in (i + 1)..(i + 50).min(sample_size) {
                let scores = StructuralComparator::compare(&functions[i], &functions[j]);
                let sim = combine(&scores, &weights);
                similarities.push(sim);
            }
        }

        if similarities.is_empty() {
            return 0.85;
        }
        similarities.sort_by(|a, b| a.total_cmp(b));

        // Find natural gap in similarity scores
        let mut best_gap = 0.0;
        let mut best_threshold = 0.85;

        for window in similarities.windows(2) {
            let gap = window[1] - window[0];
            if gap > best_gap {
                best_gap = gap;
                best_threshold = (window[0] + window[1]) / 2.0;
            }
        }

        best_threshold.clamp(0.6, 0.95)
    }
}

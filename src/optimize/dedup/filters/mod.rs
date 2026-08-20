// src/optimize/dedup/filters/mod.rs

pub mod false_positive;
pub mod threshold;

pub use false_positive::FalsePositiveFilter;
pub use threshold::{is_actionable_duplicate_candidate, ThresholdFilter, ThresholdTuner};

// src/analysis/verdict/mod.rs

pub mod label_source;
pub mod state;

// Re-export
pub use label_source::{LabelSource, VerdictState};
pub use state::VerdictEngine;

// Keep existing exports
pub use super::{Signal, SignalDirection, Verdict, VerdictConfig, VerdictStats};

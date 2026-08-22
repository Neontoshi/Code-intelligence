// src/analysis/verdict_source/mod.rs

pub mod label_source;
pub mod state;

// Re-export everything from this module
pub use label_source::{LabelSource, VerdictState};
pub use state::{
    EvidenceSource, Signal, SignalDirection, Verdict, VerdictConfig, VerdictEngine, VerdictStats,
};

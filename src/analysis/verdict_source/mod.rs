// src/analysis/verdict_source/mod.rs

pub mod label_source;
pub mod state;

pub use label_source::{LabelLevel, LabelSource, VerdictState};
pub use state::{
    EvidenceSource, Signal, SignalDirection, Verdict, VerdictConfig, VerdictEngine, VerdictStats,
};

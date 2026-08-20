// src/analysis/verdict/label_source.rs

use serde::{Deserialize, Serialize};

/// Source of a training label - critical for avoiding circularity
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LabelSource {
    /// Generated entirely by static analysis (heuristic)
    StaticHeuristic,
    /// Silver label - combination of heuristics with some confidence
    Silver,
    /// Weak label - low confidence heuristic
    Weak,
    /// Verified by a human developer
    HumanVerified,
    /// Verified by Git history (e.g., code was removed, tests passed)
    GitVerified,
    /// Verified in production (telemetry shows it's used)
    ProductionVerified,
    /// From a verified dataset (e.g., public benchmark)
    DatasetVerified,
}

impl std::fmt::Display for LabelSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LabelSource::StaticHeuristic => write!(f, "StaticHeuristic"),
            LabelSource::Silver => write!(f, "Silver"),
            LabelSource::Weak => write!(f, "Weak"),
            LabelSource::HumanVerified => write!(f, "HumanVerified"),
            LabelSource::GitVerified => write!(f, "GitVerified"),
            LabelSource::ProductionVerified => write!(f, "ProductionVerified"),
            LabelSource::DatasetVerified => write!(f, "DatasetVerified"),
        }
    }
}

impl LabelSource {
    pub fn confidence_multiplier(&self) -> f64 {
        match self {
            LabelSource::ProductionVerified => 1.0,
            LabelSource::HumanVerified => 0.98,
            LabelSource::GitVerified => 0.95,
            LabelSource::DatasetVerified => 0.92,
            LabelSource::Silver => 0.75,
            LabelSource::StaticHeuristic => 0.60,
            LabelSource::Weak => 0.40,
        }
    }

    pub fn is_verified(&self) -> bool {
        matches!(
            self,
            LabelSource::HumanVerified
                | LabelSource::GitVerified
                | LabelSource::ProductionVerified
                | LabelSource::DatasetVerified
        )
    }

    pub fn is_heuristic(&self) -> bool {
        matches!(
            self,
            LabelSource::StaticHeuristic | LabelSource::Silver | LabelSource::Weak
        )
    }
}

/// Verdict states - 5-state system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VerdictState {
    /// Definitely alive - strong evidence
    DefinitelyAlive,
    /// Probably alive - some evidence but not definitive
    ProbablyAlive,
    /// Unknown - insufficient evidence either way
    Unknown,
    /// Probably dead - some evidence but not definitive
    ProbablyDead,
    /// Definitely dead - strong evidence
    DefinitelyDead,
}

impl VerdictState {
    pub fn from_score(score: f64, dead_threshold: f64, alive_threshold: f64) -> Self {
        if score >= dead_threshold {
            VerdictState::DefinitelyDead
        } else if score >= dead_threshold * 0.8 {
            VerdictState::ProbablyDead
        } else if score <= alive_threshold {
            VerdictState::DefinitelyAlive
        } else if score <= alive_threshold * 1.5 {
            VerdictState::ProbablyAlive
        } else {
            VerdictState::Unknown
        }
    }

    pub fn confidence_label(&self) -> &'static str {
        match self {
            VerdictState::DefinitelyAlive => "🟢 DEFINITELY ALIVE",
            VerdictState::ProbablyAlive => "🟡 PROBABLY ALIVE",
            VerdictState::Unknown => "⚪ UNKNOWN",
            VerdictState::ProbablyDead => "🟠 PROBABLY DEAD",
            VerdictState::DefinitelyDead => "🔴 DEFINITELY DEAD",
        }
    }

    pub fn is_dead(&self) -> bool {
        matches!(
            self,
            VerdictState::ProbablyDead | VerdictState::DefinitelyDead
        )
    }

    pub fn is_alive(&self) -> bool {
        matches!(
            self,
            VerdictState::ProbablyAlive | VerdictState::DefinitelyAlive
        )
    }
}

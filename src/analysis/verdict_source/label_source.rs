use serde::{Deserialize, Serialize};

/// Source of a training label - critical for avoiding circularity
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum LabelSource {
    StaticHeuristic,
    Silver,
    Weak,
    HumanVerified,
    GitVerified,
    ProductionVerified,
    DatasetVerified,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LabelLevel {
    Level0 = 0,
    Level1 = 1,
    Level2 = 2,
    Level3 = 3,
    Level4 = 4,
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
    /// Training weight - whether this label should be used for ML training
    pub fn training_weight(&self) -> f64 {
        match self {
            LabelSource::ProductionVerified => 1.0,
            LabelSource::HumanVerified => 0.95,
            LabelSource::GitVerified => 0.85,
            LabelSource::DatasetVerified => 0.80,
            LabelSource::Silver => 0.30,
            LabelSource::StaticHeuristic => 0.0,
            LabelSource::Weak => 0.0,
        }
    }

    pub fn level(&self) -> LabelLevel {
        match self {
            LabelSource::ProductionVerified => LabelLevel::Level4,
            LabelSource::HumanVerified => LabelLevel::Level3,
            LabelSource::GitVerified => LabelLevel::Level2,
            LabelSource::DatasetVerified => LabelLevel::Level1,
            LabelSource::Silver => LabelLevel::Level0,
            LabelSource::StaticHeuristic | LabelSource::Weak => LabelLevel::Level0,
        }
    }

    pub fn is_trainable(&self) -> bool {
        matches!(
            self,
            LabelSource::ProductionVerified
                | LabelSource::HumanVerified
                | LabelSource::GitVerified
                | LabelSource::DatasetVerified
        )
    }

    /// Confidence multiplier for display purposes
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

    /// Whether this label is verified (not heuristic)
    pub fn is_verified(&self) -> bool {
        matches!(
            self,
            LabelSource::HumanVerified
                | LabelSource::GitVerified
                | LabelSource::ProductionVerified
                | LabelSource::DatasetVerified
        )
    }

    /// Whether this is a heuristic (not verified)
    pub fn is_heuristic(&self) -> bool {
        matches!(
            self,
            LabelSource::StaticHeuristic | LabelSource::Silver | LabelSource::Weak
        )
    }

    /// Priority for conflict resolution (higher wins)
    pub fn priority(&self) -> u8 {
        match self {
            LabelSource::ProductionVerified => 7,
            LabelSource::HumanVerified => 6,
            LabelSource::GitVerified => 5,
            LabelSource::DatasetVerified => 4,
            LabelSource::Silver => 3,
            LabelSource::StaticHeuristic => 2,
            LabelSource::Weak => 1,
        }
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

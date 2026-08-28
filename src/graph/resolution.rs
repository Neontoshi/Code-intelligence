// src/graph/resolution.rs

//! Call graph resolution with confidence tracking

use serde::{Deserialize, Serialize};

/// Resolution confidence for a call edge
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ResolutionConfidence {
    /// Exact match - verified by type system
    Exact,
    /// Inferred from context (method name matches, single candidate)
    Inferred,
    /// Heuristic match (common pattern, multiple candidates)
    Heuristic,
    /// Ambiguous - multiple possible targets
    Ambiguous,
    /// Dynamic - callback, trait dispatch, or other indirect invocation
    Dynamic,
    /// Unresolved - could not find target
    Unresolved,
}

impl ResolutionConfidence {
    pub fn confidence_score(&self) -> f64 {
        match self {
            ResolutionConfidence::Exact => 1.0,
            ResolutionConfidence::Inferred => 0.85,
            ResolutionConfidence::Heuristic => 0.65,
            ResolutionConfidence::Ambiguous => 0.40,
            ResolutionConfidence::Dynamic => 0.25,
            ResolutionConfidence::Unresolved => 0.0,
        }
    }

    pub fn is_resolved(&self) -> bool {
        !matches!(self, ResolutionConfidence::Unresolved)
    }

    pub fn is_high_confidence(&self) -> bool {
        matches!(
            self,
            ResolutionConfidence::Exact | ResolutionConfidence::Inferred
        )
    }
}

/// A resolved call target with confidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedCall {
    pub target_full_path: String,
    pub target_name: String,
    pub confidence: ResolutionConfidence,
    pub resolution_method: ResolutionMethod,
    pub source_file: String,
    pub line: usize,
}

/// How the call was resolved
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ResolutionMethod {
    /// Direct call to a known function
    Direct,
    /// Method call on self
    SelfMethod,
    /// Associated function call (Type::method)
    Associated,
    /// Constructor call (Type::new)
    Constructor,
    /// Imported function
    Import,
    /// By name (single match)
    NameMatch,
    /// In same container (impl block, class)
    ContainerMethod,
    /// Trait method (dynamic dispatch)
    TraitMethod,
    /// Callback/closure
    Callback,
    /// Higher-order function (map, and_then, etc.)
    HigherOrder,
    /// FFI/external call
    FFI,
    /// Macro-generated call
    MacroGenerated,
    /// Unresolved
    Unresolved,
}

impl ResolutionMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            ResolutionMethod::Direct => "direct",
            ResolutionMethod::SelfMethod => "self_method",
            ResolutionMethod::Associated => "associated",
            ResolutionMethod::Constructor => "constructor",
            ResolutionMethod::Import => "import",
            ResolutionMethod::NameMatch => "name_match",
            ResolutionMethod::ContainerMethod => "container_method",
            ResolutionMethod::TraitMethod => "trait_method",
            ResolutionMethod::Callback => "callback",
            ResolutionMethod::HigherOrder => "higher_order",
            ResolutionMethod::FFI => "ffi",
            ResolutionMethod::MacroGenerated => "macro_generated",
            ResolutionMethod::Unresolved => "unresolved",
        }
    }

    pub fn is_dynamic(&self) -> bool {
        matches!(
            self,
            ResolutionMethod::TraitMethod
                | ResolutionMethod::Callback
                | ResolutionMethod::HigherOrder
                | ResolutionMethod::FFI
                | ResolutionMethod::MacroGenerated
        )
    }
}

/// Call resolution result for a function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallResolution {
    pub function_full_path: String,
    pub calls: Vec<ResolvedCall>,
    pub unresolved_count: usize,
    pub total_count: usize,
    pub average_confidence: f64,
}

/// Resolution statistics for a project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionStats {
    pub total_calls: usize,
    pub resolved_calls: usize,
    pub unresolved_calls: usize,
    pub dynamic_calls: usize,
    pub exact_count: usize,
    pub inferred_count: usize,
    pub heuristic_count: usize,
    pub ambiguous_count: usize,
    pub by_method: std::collections::HashMap<String, usize>,
    pub average_confidence: f64,
    pub resolution_rate: f64,
}

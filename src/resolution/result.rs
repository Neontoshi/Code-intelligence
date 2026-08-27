// src/resolution/result.rs

use crate::resolution::symbol::SymbolId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResolutionStatus {
    Resolved,
    Ambiguous,
    External,
    Dynamic,
    Unresolved,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResolutionMethod {
    ExactSymbolId,
    LexicalScope,
    LocalSymbol,
    ContainerMember,
    ImportedSymbol,
    QualifiedSymbol,
    TypeMember,
    InheritanceResolution,
    FrameworkRoot,
    CallbackResolution,
    DynamicDispatch,
    GlobalNameFallback,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ResolutionEvidence {
    ExplicitImport,
    MatchingModule,
    MatchingSymbol,
    MatchingContainer,
    MatchingType,
    MatchingScope,
    SameFile,
    CrossFile,
    FrameworkPattern,
}

#[derive(Debug, Clone)]
pub struct ResolutionResult {
    pub status: ResolutionStatus,
    pub target: Option<SymbolId>,
    pub confidence: f64,
    pub method: Option<ResolutionMethod>,
    pub evidence: Vec<ResolutionEvidence>,
    pub candidates: Vec<ResolutionCandidate>,
}

#[derive(Debug, Clone)]
pub struct ResolutionCandidate {
    pub symbol: SymbolId,
    pub method: ResolutionMethod,
    pub confidence: f64,
    pub evidence: Vec<ResolutionEvidence>,
}

impl ResolutionResult {
    pub fn resolved(
        target: SymbolId,
        confidence: f64,
        method: ResolutionMethod,
        evidence: Vec<ResolutionEvidence>,
    ) -> Self {
        Self {
            status: ResolutionStatus::Resolved,
            target: Some(target),
            confidence,
            method: Some(method),
            evidence,
            candidates: Vec::new(),
        }
    }

    pub fn ambiguous(candidates: Vec<ResolutionCandidate>) -> Self {
        Self {
            status: ResolutionStatus::Ambiguous,
            target: None,
            confidence: 0.0,
            method: None,
            evidence: Vec::new(),
            candidates,
        }
    }

    pub fn external() -> Self {
        Self {
            status: ResolutionStatus::External,
            target: None,
            confidence: 0.0,
            method: None,
            evidence: Vec::new(),
            candidates: Vec::new(),
        }
    }

    pub fn dynamic(_reason: &str) -> Self {
        Self {
            status: ResolutionStatus::Dynamic,
            target: None,
            confidence: 0.0,
            method: None,
            evidence: vec![ResolutionEvidence::FrameworkPattern],
            candidates: Vec::new(),
        }
    }

    pub fn unresolved() -> Self {
        Self {
            status: ResolutionStatus::Unresolved,
            target: None,
            confidence: 0.0,
            method: None,
            evidence: Vec::new(),
            candidates: Vec::new(),
        }
    }
}

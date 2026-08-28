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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnresolvedReason {
    NoCandidates,
    MissingCurrentSymbol,
    ScopeMiss,
    SameFileAmbiguous,
    ImportAmbiguous,
    WildcardImportAmbiguous,
    GlobalAmbiguous,
    ContainerMiss,
    ReceiverUnbound,
    QualifiedPathMiss,
    UnsupportedCalleeShape,
    DynamicPattern,
    ExternalDependency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionDebugInfo {
    pub query: Option<String>,
    pub scope_checked: bool,
    pub same_file_candidate_count: usize,
    pub import_candidate_count: usize,
    pub wildcard_candidate_count: usize,
    pub global_candidate_count: usize,
    pub container_candidate_count: usize,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ResolutionResult {
    pub status: ResolutionStatus,
    pub target: Option<SymbolId>,
    pub confidence: f64,
    pub method: Option<ResolutionMethod>,
    pub evidence: Vec<ResolutionEvidence>,
    pub candidates: Vec<ResolutionCandidate>,
    pub reason: Option<UnresolvedReason>,
    pub debug: Option<ResolutionDebugInfo>,
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
            reason: None,
            debug: None,
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
            reason: Some(UnresolvedReason::GlobalAmbiguous),
            debug: None,
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
            reason: Some(UnresolvedReason::ExternalDependency),
            debug: None,
        }
    }

    pub fn dynamic(reason: &str) -> Self {
        Self {
            status: ResolutionStatus::Dynamic,
            target: None,
            confidence: 0.0,
            method: None,
            evidence: vec![ResolutionEvidence::FrameworkPattern],
            candidates: Vec::new(),
            reason: Some(UnresolvedReason::DynamicPattern),
            debug: Some(ResolutionDebugInfo {
                query: None,
                scope_checked: false,
                same_file_candidate_count: 0,
                import_candidate_count: 0,
                wildcard_candidate_count: 0,
                global_candidate_count: 0,
                container_candidate_count: 0,
                notes: vec![reason.to_string()],
            }),
        }
    }

    pub fn callback(reason: &str) -> Self {
        Self {
            status: ResolutionStatus::Dynamic,
            target: None,
            confidence: 0.60,
            method: Some(ResolutionMethod::CallbackResolution),
            evidence: vec![ResolutionEvidence::MatchingScope],
            candidates: Vec::new(),
            reason: Some(UnresolvedReason::DynamicPattern),
            debug: Some(ResolutionDebugInfo {
                query: None,
                scope_checked: true,
                same_file_candidate_count: 0,
                import_candidate_count: 0,
                wildcard_candidate_count: 0,
                global_candidate_count: 0,
                container_candidate_count: 0,
                notes: vec![reason.to_string()],
            }),
        }
    }

    pub fn unresolved() -> Self {
        Self::unresolved_with_reason(UnresolvedReason::NoCandidates)
    }

    pub fn unresolved_with_reason(reason: UnresolvedReason) -> Self {
        Self {
            status: ResolutionStatus::Unresolved,
            target: None,
            confidence: 0.0,
            method: None,
            evidence: Vec::new(),
            candidates: Vec::new(),
            reason: Some(reason),
            debug: None,
        }
    }

    pub fn with_debug(mut self, debug: ResolutionDebugInfo) -> Self {
        self.debug = Some(debug);
        self
    }

    pub fn with_reason(mut self, reason: UnresolvedReason) -> Self {
        self.reason = Some(reason);
        self
    }
}

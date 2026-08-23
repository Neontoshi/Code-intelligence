// tests/unit/verdict_tests.rs

//! Unit tests for the verdict system

use code_intelligence::analysis::verdict_source::label_source::{LabelSource, VerdictState};
// ⭐ Remove unused imports - keep only what's needed
use code_intelligence::analysis::training_data::{TrainingExample, TrainingLabel};
use code_intelligence::graph::call_graph::{CallGraph, FunctionNode};

// ⭐ Remove unused imports
// use code_intelligence::analysis::roots::{ReachabilityAnalyzer, RootDetectionConfig, RootDetector};
// use code_intelligence::analysis::verdict_source::state::{VerdictConfig, VerdictEngine};
// use code_intelligence::graph::call_graph::{CallEdge, CallGraph, FunctionNode};
// use code_intelligence::parser::tree_sitter::ParsedFile;

#[test]
fn test_verdict_state_from_score() {
    // Test DefinitelyDead
    let state = VerdictState::from_score(0.95, 0.92, 0.15);
    assert!(matches!(state, VerdictState::DefinitelyDead));

    // Test ProbablyDead
    let state = VerdictState::from_score(0.80, 0.92, 0.15);
    assert!(matches!(state, VerdictState::ProbablyDead));

    // Test Unknown
    let state = VerdictState::from_score(0.50, 0.92, 0.15);
    assert!(matches!(state, VerdictState::Unknown));

    // Test ProbablyAlive
    let state = VerdictState::from_score(0.20, 0.92, 0.15);
    assert!(matches!(state, VerdictState::ProbablyAlive));

    // Test DefinitelyAlive
    let state = VerdictState::from_score(0.10, 0.92, 0.15);
    assert!(matches!(state, VerdictState::DefinitelyAlive));
}

#[test]
fn test_label_source_confidence_multiplier() {
    assert_eq!(LabelSource::ProductionVerified.confidence_multiplier(), 1.0);
    assert_eq!(LabelSource::HumanVerified.confidence_multiplier(), 0.98);
    assert_eq!(LabelSource::GitVerified.confidence_multiplier(), 0.95);
    assert_eq!(LabelSource::StaticHeuristic.confidence_multiplier(), 0.60);
    assert_eq!(LabelSource::Weak.confidence_multiplier(), 0.40);

    assert!(LabelSource::HumanVerified.is_verified());
    assert!(!LabelSource::StaticHeuristic.is_verified());
}

#[test]
fn test_verdict_is_high_confidence() {
    let verdict = create_test_verdict(VerdictState::DefinitelyDead);
    assert!(verdict.is_high_confidence());

    let verdict = create_test_verdict(VerdictState::DefinitelyAlive);
    assert!(verdict.is_high_confidence());

    let verdict = create_test_verdict(VerdictState::ProbablyDead);
    assert!(!verdict.is_high_confidence());

    let verdict = create_test_verdict(VerdictState::Unknown);
    assert!(!verdict.is_high_confidence());
}

// tests/unit/verdict_tests.rs

#[test]
fn test_verdict_needs_review() {
    let verdict = create_test_verdict(VerdictState::Unknown);
    assert!(verdict.needs_review());

    // All other states should NOT need review
    let verdict = create_test_verdict(VerdictState::DefinitelyDead);
    assert!(!verdict.needs_review());

    let verdict = create_test_verdict(VerdictState::ProbablyDead);
    assert!(!verdict.needs_review());

    let verdict = create_test_verdict(VerdictState::ProbablyAlive);
    assert!(!verdict.needs_review());

    let verdict = create_test_verdict(VerdictState::DefinitelyAlive);
    assert!(!verdict.needs_review());
}

#[test]
fn test_verdict_mark_verified() {
    let mut verdict = create_test_verdict(VerdictState::ProbablyDead);
    assert!(!verdict.verified);
    assert!(verdict.verified_by.is_none());

    verdict.mark_verified("test_user");
    assert!(verdict.verified);
    assert_eq!(verdict.verified_by, Some("test_user".to_string()));
    assert!(verdict
        .evidence_sources
        .contains(&code_intelligence::analysis::verdict_source::EvidenceSource::HumanReview));
}

#[test]
fn test_training_example_label_source() {
    let func = create_test_function("test_func");
    let call_graph = CallGraph::new();

    let example = TrainingExample::new_alive(&func, &call_graph);
    assert_eq!(example.label_source, LabelSource::StaticHeuristic);
    assert!(!example.is_verified());

    let verified = TrainingExample::new_verified(
        &func,
        &call_graph,
        TrainingLabel::Alive,
        LabelSource::HumanVerified,
        "test_user",
    );
    assert_eq!(verified.label_source, LabelSource::HumanVerified);
    assert!(verified.is_verified());
}

// Helper functions
fn create_test_function(name: &str) -> FunctionNode {
    FunctionNode {
        name: name.to_string(),
        full_path: format!("test::{}", name),
        file: "test.rs".to_string(),
        line: 1,
        body_start_line: 1,
        body_end_line: 10,
        is_public: false,
        is_async: false,
        params: vec![],
        returns: vec![],
        complexity: 1.0,
        importance_score: 0.0,
        doc_comment: None,
        writes_to: vec![],
        reads_from: vec![],
        errors: vec![],
        fan_in: 0,
        fan_out: 0,
        is_cycle: false,
        depth: 0,
        layer: "core".to_string(),
        trait_impl: None,
        is_test: false,
        is_trait_method: false,
        is_trait_default: false,
    }
}

fn create_test_verdict(
    state: VerdictState,
) -> code_intelligence::analysis::verdict_source::Verdict {
    code_intelligence::analysis::verdict_source::Verdict {
        function_name: "test".to_string(),
        full_path: "test::test".to_string(),
        label: TrainingLabel::Unknown,
        state,
        confidence: 0.5,
        signals: vec![],
        dead_probability: None,
        ml_probability: None,
        static_score: Some(0.5),
        explanation: "Test verdict".to_string(),
        evidence_sources: vec![],
        verified: false,
        verified_by: None,
    }
}

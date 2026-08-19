// src/analysis/verdict.rs

use crate::analysis::dynamic_refs::DynamicReference;
use crate::analysis::roots::ReachabilityMap;
use crate::analysis::training_data::{TrainingExample, TrainingLabel};
use crate::graph::call_graph::{CallGraph, FunctionNode};
use crate::graph::traits::GraphMetrics;
use crate::ml::classifier::DeadCodeClassifier;

// Signal Types
#[derive(Debug, Clone)]
pub struct Signal {
    pub name: String,
    pub value: f64,
    pub direction: SignalDirection,
    pub weight: f64,
    pub explanation: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SignalDirection {
    SupportsDead,
    SupportsAlive,
    Neutral,
}

impl SignalDirection {
    pub fn as_str(&self) -> &'static str {
        match self {
            SignalDirection::SupportsDead => "→ DEAD",
            SignalDirection::SupportsAlive => "→ ALIVE",
            SignalDirection::Neutral => "→ NEUTRAL",
        }
    }
}

// Verdict Types

#[derive(Debug, Clone)]
pub struct Verdict {
    pub function_name: String,
    pub full_path: String,
    pub label: TrainingLabel,
    pub confidence: f64,
    pub signals: Vec<Signal>,
    pub ml_probability: Option<f64>,
    pub static_score: Option<f64>,
    pub explanation: String,
}

impl Verdict {
    pub fn is_dead(&self) -> bool {
        self.label == TrainingLabel::Dead
    }

    pub fn is_alive(&self) -> bool {
        self.label == TrainingLabel::Alive
    }

    pub fn needs_review(&self) -> bool {
        self.label == TrainingLabel::Unknown
    }

    pub fn format_explanation(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!("Function: {}\n", self.function_name));
        output.push_str(&format!("Verdict: {:?}\n", self.label));
        output.push_str(&format!("Confidence: {:.1}%\n\n", self.confidence * 100.0));

        output.push_str("Signals:\n");
        for signal in &self.signals {
            output.push_str(&format!(
                "  - {}: {:.2} {}\n",
                signal.name,
                signal.value,
                signal.direction.as_str()
            ));
        }

        if let Some(ml) = self.ml_probability {
            output.push_str(&format!("\nML Probability: {:.1}%", ml * 100.0));
        }

        if let Some(static_score) = self.static_score {
            output.push_str(&format!("\nStatic Score: {:.1}%", static_score * 100.0));
        }

        output.push_str(&format!("\n\nExplanation: {}\n", self.explanation));

        output
    }
}

// Verdict Engine

#[derive(Debug, Clone)]
pub struct VerdictConfig {
    pub dead_threshold: f64,
    pub alive_threshold: f64,
    pub enable_ml: bool,
    pub enable_static: bool,
}

impl Default for VerdictConfig {
    fn default() -> Self {
        Self {
            dead_threshold: 0.92,
            alive_threshold: 0.15,
            enable_ml: true,
            enable_static: true,
        }
    }
}

pub struct VerdictEngine {
    config: VerdictConfig,
    ml_model: Option<DeadCodeClassifier>,
    dynamic_refs: Option<Vec<DynamicReference>>,
}

impl VerdictEngine {
    pub fn new(config: VerdictConfig) -> Self {
        Self {
            config,
            ml_model: None,
            dynamic_refs: None,
        }
    }

    pub fn with_model_thresholds(self, _model: &DeadCodeClassifier) -> Self {
        self
    }

    pub fn with_dead_threshold(mut self, threshold: f64) -> Self {
        self.config.dead_threshold = threshold;
        self
    }

    pub fn with_alive_threshold(mut self, threshold: f64) -> Self {
        self.config.alive_threshold = threshold;
        self
    }

    pub fn with_ml(mut self, model: DeadCodeClassifier) -> Self {
        self.ml_model = Some(model);
        self.config.enable_ml = true;
        self
    }

    pub fn with_dynamic_refs(mut self, refs: Vec<DynamicReference>) -> Self {
        self.dynamic_refs = Some(refs);
        self
    }

    pub fn evaluate_function(
        &self,
        func: &FunctionNode,
        call_graph: &CallGraph,
        reachability: &ReachabilityMap,
    ) -> Verdict {
        let mut signals = Vec::new();
        let mut static_score = 0.0;

        let static_signals = self.collect_static_signals(func, reachability);
        for signal in &static_signals {
            if signal.direction == SignalDirection::SupportsDead {
                static_score += signal.weight;
            } else if signal.direction == SignalDirection::SupportsAlive {
                static_score -= signal.weight;
            }
        }
        signals.extend(static_signals);

        let total_weight: f64 = signals.iter().map(|s| s.weight).sum();
        let normalized_static = if total_weight > 0.0 {
            (static_score / total_weight + 1.0) / 2.0
        } else {
            0.5
        };
        let normalized_static = normalized_static.clamp(0.0, 1.0);

        let ml_probability = if self.config.enable_ml {
            if let Some(model) = &self.ml_model {
                use crate::analysis::training_data::FunctionFeatures;
                let features = FunctionFeatures::from_function(func, call_graph);
                let example = TrainingExample {
                    function_name: func.name.clone(),
                    full_path: func.full_path.clone(),
                    file: func.file.clone(),
                    language: TrainingExample::detect_language(&func.file),
                    features,
                    label: TrainingLabel::Unknown,
                    confidence: 0.0,
                    source: "ml".to_string(),
                    repository_id: None,
                    commit_hash: None,
                    dataset_split: None,
                    label_reason: Some("ml".to_string()),
                    label_version: Some(1),
                };
                let alive_prob = model.predict_probability(&example);
                let dead_prob = 1.0 - alive_prob;

                signals.push(Signal {
                    name: "ml_prediction".to_string(),
                    value: dead_prob,
                    direction: if dead_prob > 0.5 {
                        SignalDirection::SupportsDead
                    } else {
                        SignalDirection::SupportsAlive
                    },
                    weight: 0.4,
                    explanation: format!(
                        "ML model predicts {:.1}% chance of being dead",
                        dead_prob * 100.0
                    ),
                });
                Some(dead_prob)
            } else {
                None
            }
        } else {
            None
        };

        let combined_score = if let Some(ml) = ml_probability {
            normalized_static * 0.6 + ml * 0.4
        } else {
            normalized_static
        };

        let (label, confidence) = if combined_score >= self.config.dead_threshold {
            (TrainingLabel::Dead, combined_score)
        } else if combined_score <= self.config.alive_threshold {
            (TrainingLabel::Alive, 1.0 - combined_score)
        } else {
            (TrainingLabel::Unknown, combined_score)
        };

        let explanation = self.generate_explanation(
            func,
            &signals,
            label.clone(),
            combined_score,
            ml_probability,
        );

        Verdict {
            function_name: func.name.clone(),
            full_path: func.full_path.clone(),
            label,
            confidence,
            signals,
            ml_probability,
            static_score: Some(normalized_static),
            explanation,
        }
    }

    pub fn evaluate_all(
        &self,
        call_graph: &CallGraph,
        reachability: &ReachabilityMap,
    ) -> Vec<Verdict> {
        let mut verdicts = Vec::new();
        let total_nodes = call_graph.node_count();

        let max_nodes = 2000;
        if total_nodes > max_nodes {
            eprintln!(
                "⚠️ Large call graph ({} nodes). Limiting evaluation to {} nodes for safety.",
                total_nodes, max_nodes
            );
        }

        let indices: Vec<_> = call_graph.node_indices().take(max_nodes).collect();

        for idx in indices {
            let func = &call_graph[idx];
            let verdict = self.evaluate_function(func, call_graph, reachability);
            verdicts.push(verdict);
        }

        // Use total_cmp - never panics on NaN
        verdicts.sort_by(|a, b| b.confidence.total_cmp(&a.confidence));
        verdicts
    }

    fn collect_static_signals(
        &self,
        func: &FunctionNode,
        reachability: &ReachabilityMap,
    ) -> Vec<Signal> {
        let mut signals = Vec::new();

        let fan_in_value = func.fan_in as f64 / 10.0;
        if func.fan_in == 0 {
            signals.push(Signal {
                name: "fan_in".to_string(),
                value: 0.0,
                direction: SignalDirection::SupportsDead,
                weight: 0.4,
                explanation: "No callers found".to_string(),
            });
        } else {
            signals.push(Signal {
                name: "fan_in".to_string(),
                value: fan_in_value.min(1.0),
                direction: SignalDirection::SupportsAlive,
                weight: 0.4,
                explanation: format!("{} callers", func.fan_in),
            });
        }

        let is_reachable = reachability.is_reachable(&func.full_path);
        if is_reachable {
            signals.push(Signal {
                name: "reachability".to_string(),
                value: 1.0,
                direction: SignalDirection::SupportsAlive,
                weight: 0.3,
                explanation: "Reachable from entry points".to_string(),
            });
        } else {
            signals.push(Signal {
                name: "reachability".to_string(),
                value: 0.0,
                direction: SignalDirection::SupportsDead,
                weight: 0.3,
                explanation: "Unreachable from entry points".to_string(),
            });
        }

        if func.is_public {
            signals.push(Signal {
                name: "is_public".to_string(),
                value: 1.0,
                direction: SignalDirection::SupportsAlive,
                weight: 0.2,
                explanation: "Public API".to_string(),
            });
        } else {
            signals.push(Signal {
                name: "is_public".to_string(),
                value: 0.0,
                direction: SignalDirection::SupportsDead,
                weight: 0.2,
                explanation: "Private function".to_string(),
            });
        }

        if func.trait_impl.is_some() {
            signals.push(Signal {
                name: "trait_impl".to_string(),
                value: 1.0,
                direction: SignalDirection::SupportsAlive,
                weight: 0.15,
                explanation: "Trait implementation".to_string(),
            });
        }

        if func.complexity > 10.0 {
            signals.push(Signal {
                name: "complexity".to_string(),
                value: (func.complexity / 50.0).min(1.0),
                direction: SignalDirection::SupportsAlive,
                weight: 0.1,
                explanation: format!("High complexity ({:.1})", func.complexity),
            });
        }

        if func.doc_comment.is_some() {
            signals.push(Signal {
                name: "documentation".to_string(),
                value: 1.0,
                direction: SignalDirection::SupportsAlive,
                weight: 0.1,
                explanation: "Has documentation".to_string(),
            });
        }

        if let Some(dynamic_refs) = &self.dynamic_refs {
            let is_dynamically_referenced = dynamic_refs.iter().any(|r| {
                r.target_function
                    .as_ref()
                    .map(|f| f == &func.name)
                    .unwrap_or(false)
                    || r.target_pattern.contains(&func.name)
            });

            if is_dynamically_referenced {
                signals.push(Signal {
                    name: "dynamic_reference".to_string(),
                    value: 1.0,
                    direction: SignalDirection::SupportsAlive,
                    weight: 0.4,
                    explanation: "Referenced dynamically (reflection/callback)".to_string(),
                });
            }
        }

        signals
    }

    fn generate_explanation(
        &self,
        _func: &FunctionNode,
        signals: &[Signal],
        label: TrainingLabel,
        confidence: f64,
        ml_probability: Option<f64>,
    ) -> String {
        let mut parts = Vec::new();

        match label {
            TrainingLabel::Dead => {
                parts.push(format!(
                    "This function is likely dead ({:.1}% confidence).",
                    confidence * 100.0
                ));

                let dead_signals: Vec<_> = signals
                    .iter()
                    .filter(|s| s.direction == SignalDirection::SupportsDead)
                    .collect();

                if !dead_signals.is_empty() {
                    let reasons: Vec<_> =
                        dead_signals.iter().map(|s| s.explanation.clone()).collect();
                    parts.push(format!("Reasons: {}", reasons.join(", ")));
                }

                if let Some(ml) = ml_probability {
                    parts.push(format!(
                        "ML model predicts {:.1}% probability of being dead.",
                        ml * 100.0
                    ));
                }
            }
            TrainingLabel::Alive => {
                parts.push(format!(
                    "This function is alive ({:.1}% confidence).",
                    confidence * 100.0
                ));

                let alive_signals: Vec<_> = signals
                    .iter()
                    .filter(|s| s.direction == SignalDirection::SupportsAlive)
                    .collect();

                if !alive_signals.is_empty() {
                    let reasons: Vec<_> = alive_signals
                        .iter()
                        .map(|s| s.explanation.clone())
                        .collect();
                    parts.push(format!("Evidence: {}", reasons.join(", ")));
                }
            }
            TrainingLabel::Unknown => {
                parts.push(
                    "Insufficient evidence to determine if this function is dead or alive."
                        .to_string(),
                );
                parts.push("Review required.".to_string());
            }
        }

        parts.join(" ")
    }

    pub fn filter_dead<'a>(&self, verdicts: &'a [Verdict]) -> Vec<&'a Verdict> {
        verdicts.iter().filter(|v| v.is_dead()).collect()
    }

    pub fn filter_alive<'a>(&self, verdicts: &'a [Verdict]) -> Vec<&'a Verdict> {
        verdicts.iter().filter(|v| v.is_alive()).collect()
    }

    pub fn filter_unknown<'a>(&self, verdicts: &'a [Verdict]) -> Vec<&'a Verdict> {
        verdicts.iter().filter(|v| v.needs_review()).collect()
    }

    pub fn default_with_threshold(threshold: f64) -> Self {
        let mut config = VerdictConfig::default();
        config.dead_threshold = threshold;
        Self::new(config)
    }

    pub fn with_ml_model(threshold: f64, model: DeadCodeClassifier) -> Self {
        let mut engine = Self::default_with_threshold(threshold);
        engine = engine.with_ml(model);
        engine
    }

    pub fn stats(&self, verdicts: &[Verdict]) -> VerdictStats {
        let dead = verdicts.iter().filter(|v| v.is_dead()).count();
        let alive = verdicts.iter().filter(|v| v.is_alive()).count();
        let unknown = verdicts.iter().filter(|v| v.needs_review()).count();

        let avg_confidence: f64 =
            verdicts.iter().map(|v| v.confidence).sum::<f64>() / verdicts.len() as f64;

        VerdictStats {
            total: verdicts.len(),
            dead,
            alive,
            unknown,
            avg_confidence,
        }
    }
}

#[derive(Debug, Clone)]
pub struct VerdictStats {
    pub total: usize,
    pub dead: usize,
    pub alive: usize,
    pub unknown: usize,
    pub avg_confidence: f64,
}

impl VerdictStats {
    pub fn dead_ratio(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.dead as f64 / self.total as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::roots::{ReachabilityAnalyzer, RootDetectionConfig, RootDetector};
    use crate::graph::call_graph::{CallEdge, CallGraph, FunctionNode};
    use crate::parser::tree_sitter::ParsedFile;

    fn create_test_graph() -> (CallGraph, ReachabilityMap) {
        let mut graph = CallGraph::new();

        let entry = FunctionNode {
            name: "main".to_string(),
            full_path: "test::main".to_string(),
            file: "test.rs".to_string(),
            line: 1,
            body_start_line: 1,
            body_end_line: 1,
            is_public: true,
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
        };

        let used = FunctionNode {
            name: "used_function".to_string(),
            full_path: "test::used_function".to_string(),
            file: "test.rs".to_string(),
            line: 10,
            body_start_line: 10,
            body_end_line: 10,
            is_public: false,
            is_async: false,
            params: vec![],
            returns: vec![],
            complexity: 2.0,
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
        };

        let unused = FunctionNode {
            name: "unused_function".to_string(),
            full_path: "test::unused_function".to_string(),
            file: "test.rs".to_string(),
            line: 20,
            body_start_line: 20,
            body_end_line: 20,
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
        };

        let entry_idx = graph.add_function(entry);
        let used_idx = graph.add_function(used);
        let _unused_idx = graph.add_function(unused);

        graph.add_call(
            entry_idx,
            used_idx,
            CallEdge {
                call_type: "direct".to_string(),
                line: 2,
            },
        );

        graph.calculate_fan_metrics();

        let files: Vec<ParsedFile> = vec![];
        let config = RootDetectionConfig::default();
        let root_set = RootDetector::detect_roots(&graph, &files, &config);
        let reachability = ReachabilityAnalyzer::compute_reachability(&graph, &root_set);

        (graph, reachability)
    }

    #[test]
    fn test_verdict_engine_evaluate_all() {
        let (graph, reachability) = create_test_graph();

        let mut config = VerdictConfig::default();
        config.enable_ml = false;
        config.enable_static = true;
        config.dead_threshold = 0.60;
        config.alive_threshold = 0.40;

        let engine = VerdictEngine::new(config);
        let verdicts = engine.evaluate_all(&graph, &reachability);

        assert_eq!(verdicts.len(), 3, "Should have 3 verdicts");

        let dead_count = verdicts
            .iter()
            .filter(|v| v.label == TrainingLabel::Dead)
            .count();
        assert_eq!(dead_count, 1, "Should find exactly 1 dead function");

        let alive_count = verdicts
            .iter()
            .filter(|v| v.label == TrainingLabel::Alive)
            .count();
        assert!(alive_count >= 1, "Should find at least 1 alive function");
    }

    #[test]
    fn test_filter_dead_verdicts() {
        let (graph, reachability) = create_test_graph();

        let mut config = VerdictConfig::default();
        config.enable_ml = false;
        config.enable_static = true;
        config.dead_threshold = 0.60;
        config.alive_threshold = 0.40;

        let engine = VerdictEngine::new(config);
        let verdicts = engine.evaluate_all(&graph, &reachability);
        let dead = engine.filter_dead(&verdicts);

        assert_eq!(dead.len(), 1, "Should find exactly 1 dead function");
        assert_eq!(
            dead[0].function_name, "unused_function",
            "Dead function should be unused_function"
        );
    }

    #[test]
    fn test_verdict_stats() {
        let (graph, reachability) = create_test_graph();

        let mut config = VerdictConfig::default();
        config.enable_ml = false;
        config.enable_static = true;
        config.dead_threshold = 0.60;
        config.alive_threshold = 0.40;

        let engine = VerdictEngine::new(config);
        let verdicts = engine.evaluate_all(&graph, &reachability);
        let stats = engine.stats(&verdicts);

        assert_eq!(stats.total, 3, "Should have 3 total verdicts");
        assert_eq!(stats.dead, 1, "Should have 1 dead verdict");
        assert!(stats.alive >= 1, "Should have at least 1 alive verdict");
        assert!(
            stats.avg_confidence >= 0.0,
            "Average confidence should be >= 0"
        );
    }

    #[test]
    fn test_verdict_engine_with_threshold() {
        let (graph, reachability) = create_test_graph();

        let config_high = VerdictConfig {
            enable_ml: false,
            enable_static: true,
            dead_threshold: 0.95,
            alive_threshold: 0.40,
        };
        let engine_high = VerdictEngine::new(config_high);
        let verdicts_high = engine_high.evaluate_all(&graph, &reachability);
        let dead_high = verdicts_high
            .iter()
            .filter(|v| v.label == TrainingLabel::Dead)
            .count();

        let config_low = VerdictConfig {
            enable_ml: false,
            enable_static: true,
            dead_threshold: 0.50,
            alive_threshold: 0.40,
        };
        let engine_low = VerdictEngine::new(config_low);
        let verdicts_low = engine_low.evaluate_all(&graph, &reachability);
        let dead_low = verdicts_low
            .iter()
            .filter(|v| v.label == TrainingLabel::Dead)
            .count();

        assert!(
            dead_high <= dead_low,
            "Higher threshold should find fewer or equal dead functions"
        );
    }

    #[test]
    fn test_verdict_signals_collection() {
        let (graph, reachability) = create_test_graph();

        let mut config = VerdictConfig::default();
        config.enable_ml = false;
        config.enable_static = true;

        let engine = VerdictEngine::new(config);

        let unused_idx = graph
            .node_indices()
            .find(|idx| graph[*idx].name == "unused_function")
            .unwrap();
        let unused_func = &graph[unused_idx];
        let verdict = engine.evaluate_function(unused_func, &graph, &reachability);

        let has_dead_signal = verdict
            .signals
            .iter()
            .any(|s| s.direction == SignalDirection::SupportsDead);
        assert!(has_dead_signal, "Unused function should have dead signals");

        let used_idx = graph
            .node_indices()
            .find(|idx| graph[*idx].name == "used_function")
            .unwrap();
        let used_func = &graph[used_idx];
        let verdict = engine.evaluate_function(used_func, &graph, &reachability);

        let has_alive_signal = verdict
            .signals
            .iter()
            .any(|s| s.direction == SignalDirection::SupportsAlive);
        assert!(has_alive_signal, "Used function should have alive signals");
    }
}

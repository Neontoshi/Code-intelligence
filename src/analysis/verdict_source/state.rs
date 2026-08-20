// src/analysis/verdict/state.rs

use crate::analysis::dynamic_refs::DynamicReference;
use crate::analysis::roots::ReachabilityMap;
use crate::analysis::training_data::{TrainingExample, TrainingLabel};
use crate::analysis::verdict::{EvidenceSource, Signal, SignalDirection, Verdict};
use crate::analysis::verdict_source::label_source::VerdictState;
use crate::graph::call_graph::{CallGraph, FunctionNode};
use crate::graph::traits::GraphMetrics;
use crate::ml::classifier::DeadCodeClassifier;

#[derive(Debug, Clone)]
pub struct VerdictConfig {
    pub dead_threshold: f64,
    pub alive_threshold: f64,
    pub enable_ml: bool,
    pub enable_static: bool,
    pub model_version: Option<String>,
}

impl Default for VerdictConfig {
    fn default() -> Self {
        Self {
            dead_threshold: 0.92,
            alive_threshold: 0.15,
            enable_ml: true,
            enable_static: true,
            model_version: None,
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
        let mut evidence_sources = Vec::new();
        let mut static_score = 0.0;

        // 1. Collect static signals
        let static_signals = self.collect_static_signals(func, reachability);
        for signal in &static_signals {
            if signal.direction == SignalDirection::SupportsDead {
                static_score += signal.weight;
            } else if signal.direction == SignalDirection::SupportsAlive {
                static_score -= signal.weight;
            }
        }
        signals.extend(static_signals);

        // Track evidence sources from static analysis
        evidence_sources.push(EvidenceSource::StaticReachability);
        evidence_sources.push(EvidenceSource::CallGraph);

        // 2. Normalize static score
        let total_weight: f64 = signals.iter().map(|s| s.weight).sum();
        let normalized_static = if total_weight > 0.0 {
            (static_score / total_weight + 1.0) / 2.0
        } else {
            0.5
        };
        let normalized_static = normalized_static.clamp(0.0, 1.0);

        // 3. ML prediction (if enabled)
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
                    label_source: crate::analysis::verdict_source::LabelSource::StaticHeuristic,
                    generated_by_model: self.config.model_version.clone(),
                    verified_by: None,
                    created_at: Some(chrono::Utc::now().timestamp()),
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

                evidence_sources.push(EvidenceSource::MLModel(
                    self.config
                        .model_version
                        .clone()
                        .unwrap_or_else(|| "unknown".to_string()),
                ));
                Some(dead_prob)
            } else {
                None
            }
        } else {
            None
        };

        // 4. Dynamic references (if available)
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
                evidence_sources.push(EvidenceSource::DynamicRefs);
            }
        }

        // 5. Git history (if available)
        // This would be added by the caller

        // 6. Combined score
        let combined_score = if let Some(ml) = ml_probability {
            normalized_static * 0.6 + ml * 0.4
        } else {
            normalized_static
        };

        // 7. Determine 5-state verdict
        let state = VerdictState::from_score(
            combined_score,
            self.config.dead_threshold,
            self.config.alive_threshold,
        );

        // 8. Determine label
        let label = match state {
            VerdictState::DefinitelyDead | VerdictState::ProbablyDead => TrainingLabel::Dead,
            VerdictState::DefinitelyAlive | VerdictState::ProbablyAlive => TrainingLabel::Alive,
            VerdictState::Unknown => TrainingLabel::Unknown,
        };

        let confidence = match state {
            VerdictState::DefinitelyDead | VerdictState::DefinitelyAlive => combined_score,
            VerdictState::ProbablyDead | VerdictState::ProbablyAlive => combined_score * 0.85,
            VerdictState::Unknown => 0.5,
        };

        // 9. Generate explanation
        let explanation = self.generate_explanation(
            func,
            &signals,
            state.clone(),
            combined_score,
            ml_probability,
        );

        Verdict {
            function_name: func.name.clone(),
            full_path: func.full_path.clone(),
            label,
            state,
            confidence,
            signals,
            ml_probability,
            static_score: Some(normalized_static),
            explanation,
            evidence_sources,
            verified: false,
            verified_by: None,
        }
    }

    pub fn evaluate_all(
        &self,
        call_graph: &CallGraph,
        reachability: &ReachabilityMap,
    ) -> Vec<Verdict> {
        let mut verdicts = Vec::with_capacity(call_graph.node_count());

        for idx in call_graph.node_indices() {
            let func = &call_graph[idx];
            let verdict = self.evaluate_function(func, call_graph, reachability);
            verdicts.push(verdict);
        }

        // Sort by confidence (high to low)
        verdicts.sort_by(|a, b| b.confidence.total_cmp(&a.confidence));
        verdicts
    }

    fn collect_static_signals(
        &self,
        func: &FunctionNode,
        reachability: &ReachabilityMap,
    ) -> Vec<Signal> {
        let mut signals = Vec::new();

        // Fan-in signal
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

        // Reachability signal
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

        // Public API signal
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

        // Trait implementation signal
        if func.trait_impl.is_some() {
            signals.push(Signal {
                name: "trait_impl".to_string(),
                value: 1.0,
                direction: SignalDirection::SupportsAlive,
                weight: 0.15,
                explanation: "Trait implementation".to_string(),
            });
        }

        // Complexity signal (high complexity = more likely alive)
        if func.complexity > 10.0 {
            signals.push(Signal {
                name: "complexity".to_string(),
                value: (func.complexity / 50.0).min(1.0),
                direction: SignalDirection::SupportsAlive,
                weight: 0.1,
                explanation: format!("High complexity ({:.1})", func.complexity),
            });
        }

        // Documentation signal
        if func.doc_comment.is_some() {
            signals.push(Signal {
                name: "documentation".to_string(),
                value: 1.0,
                direction: SignalDirection::SupportsAlive,
                weight: 0.1,
                explanation: "Has documentation".to_string(),
            });
        }

        signals
    }

    fn generate_explanation(
        &self,
        _func: &FunctionNode,
        signals: &[Signal],
        state: VerdictState,
        _confidence: f64,
        ml_probability: Option<f64>,
    ) -> String {
        let mut parts = Vec::new();

        parts.push(state.confidence_label().to_string());

        match state {
            VerdictState::DefinitelyDead | VerdictState::ProbablyDead => {
                let dead_signals: Vec<_> = signals
                    .iter()
                    .filter(|s| s.direction == SignalDirection::SupportsDead)
                    .collect();

                if !dead_signals.is_empty() {
                    let reasons: Vec<_> =
                        dead_signals.iter().map(|s| s.explanation.clone()).collect();
                    parts.push(format!("Evidence: {}", reasons.join(", ")));
                }

                if let Some(ml) = ml_probability {
                    parts.push(format!("ML confidence: {:.1}%", ml * 100.0));
                }
            }
            VerdictState::DefinitelyAlive | VerdictState::ProbablyAlive => {
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
            VerdictState::Unknown => {
                parts.push(
                    "Insufficient evidence to determine if this function is dead or alive."
                        .to_string(),
                );
                parts.push("Review required.".to_string());
            }
        }

        parts.join(" | ")
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

    pub fn filter_high_confidence<'a>(&self, verdicts: &'a [Verdict]) -> Vec<&'a Verdict> {
        verdicts.iter().filter(|v| v.is_high_confidence()).collect()
    }

    pub fn stats(&self, verdicts: &[Verdict]) -> VerdictStats {
        let dead = verdicts.iter().filter(|v| v.is_dead()).count();
        let alive = verdicts.iter().filter(|v| v.is_alive()).count();
        let unknown = verdicts.iter().filter(|v| v.needs_review()).count();
        let high_confidence = verdicts.iter().filter(|v| v.is_high_confidence()).count();

        let avg_confidence: f64 =
            verdicts.iter().map(|v| v.confidence).sum::<f64>() / verdicts.len() as f64;

        VerdictStats {
            total: verdicts.len(),
            dead,
            alive,
            unknown,
            high_confidence,
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
    pub high_confidence: usize,
    pub avg_confidence: f64,
}

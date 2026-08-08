// src/analysis/verdict.rs

//! Unified verdict engine combining static evidence + ML predictions
//!
//! This module provides a single source of truth for determining if a function
//! is dead, alive, or needs review.

use crate::analysis::dynamic_refs::DynamicReference;
use crate::analysis::roots::ReachabilityMap;
use crate::analysis::training_data::{TrainingExample, TrainingLabel};
use crate::graph::call_graph::{CallGraph, FunctionNode};
use crate::ml::classifier::DeadCodeClassifier;

// ============================================================================
// Signal Types
// ============================================================================

/// A single piece of evidence about a function
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

// ============================================================================
// Verdict Types
// ============================================================================

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

    /// Generate a human-readable explanation
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

// ============================================================================
// Verdict Engine
// ============================================================================

#[derive(Debug, Clone)]
pub struct VerdictConfig {
    /// Minimum confidence to label as Dead
    pub dead_threshold: f64,
    /// Minimum confidence to label as Alive
    pub alive_threshold: f64,
    /// Whether to use ML predictions
    pub enable_ml: bool,
    /// Whether to use static analysis
    pub enable_static: bool,
}

impl Default for VerdictConfig {
    fn default() -> Self {
        Self {
            dead_threshold: 0.92,
            alive_threshold: 0.85,
            enable_ml: true,
            enable_static: true,
        }
    }
}

pub struct VerdictEngine {
    config: VerdictConfig,
    ml_model: Option<DeadCodeClassifier>,
    dynamic_refs: Option<Vec<DynamicReference>>, // ⭐ NEW
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

    /// Generate a verdict for a single function
    pub fn evaluate_function(
        &self,
        func: &FunctionNode,
        call_graph: &CallGraph,
        reachability: &ReachabilityMap,
    ) -> Verdict {
        let mut signals = Vec::new();
        let mut static_score = 0.0;
        let mut signal_count = 0;

        // 1. Static Analysis Signals
        let static_signals = self.collect_static_signals(func, reachability);
        for signal in &static_signals {
            if signal.direction == SignalDirection::SupportsDead {
                static_score += signal.weight;
            } else if signal.direction == SignalDirection::SupportsAlive {
                static_score -= signal.weight;
            }
            signal_count += 1;
        }
        signals.extend(static_signals);

        // Normalize static score to 0-1 range
        let normalized_static = if signal_count > 0 {
            (static_score / signal_count as f64 + 1.0) / 2.0
        } else {
            0.5
        };
        let normalized_static = normalized_static.clamp(0.0, 1.0);

        // 2. ML Prediction
        let ml_probability = if self.config.enable_ml {
            if let Some(model) = &self.ml_model {
                // Create training example
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
                };
                let prob = model.predict_probability(&example);
                signals.push(Signal {
                    name: "ml_prediction".to_string(),
                    value: prob,
                    direction: if prob > 0.5 {
                        SignalDirection::SupportsDead
                    } else {
                        SignalDirection::SupportsAlive
                    },
                    weight: 0.4,
                    explanation: format!(
                        "ML model predicts {:.1}% chance of being dead",
                        prob * 100.0
                    ),
                });
                Some(prob)
            } else {
                None
            }
        } else {
            None
        };

        // 3. Combine evidence
        let combined_score = if let Some(ml) = ml_probability {
            // Weighted average: 60% static, 40% ML
            normalized_static * 0.6 + ml * 0.4
        } else {
            normalized_static
        };

        // 4. Determine label and confidence
        let (label, confidence) = if combined_score >= self.config.dead_threshold {
            (TrainingLabel::Dead, combined_score)
        } else if combined_score <= self.config.alive_threshold {
            (TrainingLabel::Alive, 1.0 - combined_score)
        } else {
            (TrainingLabel::Unknown, combined_score)
        };

        // 5. Generate explanation
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

    /// Evaluate all functions in the call graph
    pub fn evaluate_all(
        &self,
        call_graph: &CallGraph,
        reachability: &ReachabilityMap,
    ) -> Vec<Verdict> {
        let mut verdicts = Vec::new();

        for idx in call_graph.node_indices() {
            let func = &call_graph[idx];
            let verdict = self.evaluate_function(func, call_graph, reachability);
            verdicts.push(verdict);
        }

        // Sort by confidence (highest first)
        verdicts.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

        verdicts
    }

    // ========================================================================
    // Helper Methods
    // ========================================================================

    fn collect_static_signals(
        &self,
        func: &FunctionNode,
        reachability: &ReachabilityMap,
    ) -> Vec<Signal> {
        let mut signals = Vec::new();

        // 1. Fan-in (callers)
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

        // 2. Reachability
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

        // 3. Public/Private
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

        // 4. Trait Implementation
        if func.trait_impl.is_some() {
            signals.push(Signal {
                name: "trait_impl".to_string(),
                value: 1.0,
                direction: SignalDirection::SupportsAlive,
                weight: 0.15,
                explanation: "Trait implementation".to_string(),
            });
        }

        // 5. Complexity (high complexity suggests it might be used)
        if func.complexity > 10.0 {
            signals.push(Signal {
                name: "complexity".to_string(),
                value: (func.complexity / 50.0).min(1.0),
                direction: SignalDirection::SupportsAlive,
                weight: 0.1,
                explanation: format!("High complexity ({:.1})", func.complexity),
            });
        }

        // 6. Documentation
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
                r.source_function
                    .as_ref()
                    .map(|f| f == &func.name)
                    .unwrap_or(false)
            });

            if is_dynamically_referenced {
                signals.push(Signal {
                    name: "dynamic_reference".to_string(),
                    value: 1.0,
                    direction: SignalDirection::SupportsAlive,
                    weight: 0.3,
                    explanation: "Referenced dynamically (reflection/callback)".to_string(),
                });
            }
        }

        signals
    }

    fn generate_explanation(
        &self,
        _func: &FunctionNode, // Added underscore to fix unused variable warning
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
                    parts.push(format!("ML model agrees: {:.1}% probability.", ml * 100.0));
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

    /// Filter verdicts by label - fixed lifetime
    pub fn filter_dead<'a>(&self, verdicts: &'a [Verdict]) -> Vec<&'a Verdict> {
        verdicts.iter().filter(|v| v.is_dead()).collect()
    }

    /// Filter verdicts by label - fixed lifetime
    pub fn filter_alive<'a>(&self, verdicts: &'a [Verdict]) -> Vec<&'a Verdict> {
        verdicts.iter().filter(|v| v.is_alive()).collect()
    }

    /// Filter verdicts by label - fixed lifetime
    pub fn filter_unknown<'a>(&self, verdicts: &'a [Verdict]) -> Vec<&'a Verdict> {
        verdicts.iter().filter(|v| v.needs_review()).collect()
    }

    /// Get verdict statistics
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

// src/analysis/verdict_source/state.rs

use crate::analysis::dead_code::filters::is_never_dead;
use crate::analysis::dynamic_refs::DynamicReference;
use crate::analysis::roots::ReachabilityMap;
use crate::analysis::training_data::{TrainingExample, TrainingLabel};
use crate::analysis::verdict_source::label_source::VerdictState;
use crate::graph::call_graph::{CallGraph, FunctionNode};
use crate::graph::traits::GraphMetrics;
use crate::ml::classifier::DeadCodeClassifier;

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

#[derive(Debug, Clone)]
pub struct Verdict {
    pub function_name: String,
    pub full_path: String,
    pub label: TrainingLabel,
    pub state: VerdictState,
    pub confidence: f64,
    pub dead_probability: Option<f64>,
    pub signals: Vec<Signal>,
    pub ml_probability: Option<f64>,
    pub static_score: Option<f64>,
    pub explanation: String,
    pub evidence_sources: Vec<EvidenceSource>,
    pub verified: bool,
    pub verified_by: Option<String>,
    pub provenance: VerdictProvenance,
}

/// Provenance information for a verdict
#[derive(Debug, Clone)]
pub struct VerdictProvenance {
    /// Version of the analysis engine
    pub analysis_version: String,
    /// Model version used (if any)
    pub model_version: Option<String>,
    /// Git commit SHA (if available)
    pub commit_sha: Option<String>,
    /// Feature schema version
    pub feature_schema_version: u32,
    /// When the analysis was performed
    pub analysis_timestamp: i64,
    /// How long the analysis took (in seconds)
    pub analysis_duration_secs: Option<f64>,
    /// Which pipeline stages were used
    pub stages_used: Vec<String>,
    /// Whether ML was enabled
    pub ml_enabled: bool,
    /// Whether static analysis was enabled
    pub static_enabled: bool,
    /// Model file path (if any)
    pub model_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EvidenceSource {
    StaticReachability,
    CallGraph,
    DynamicRefs,
    MLModel(String),
    GitHistory,
    HumanReview,
    ProductionTelemetry,
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

    pub fn is_high_confidence(&self) -> bool {
        matches!(
            self.state,
            VerdictState::DefinitelyAlive | VerdictState::DefinitelyDead
        )
    }

    pub fn get_dead_probability(&self) -> Option<f64> {
        self.dead_probability
    }

    pub fn get_confidence(&self) -> f64 {
        self.confidence
    }

    pub fn mark_verified(&mut self, verified_by: &str) {
        self.verified = true;
        self.verified_by = Some(verified_by.to_string());
        self.evidence_sources.push(EvidenceSource::HumanReview);
    }

    pub fn format_state(&self) -> String {
        self.state.confidence_label().to_string()
    }

    /// Format provenance as a string
    pub fn format_provenance(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!(
            "Analysis Version: {}\n",
            self.provenance.analysis_version
        ));

        if let Some(model_ver) = &self.provenance.model_version {
            s.push_str(&format!("Model Version: {}\n", model_ver));
        }

        if let Some(commit) = &self.provenance.commit_sha {
            s.push_str(&format!("Commit: {}\n", &commit[..8]));
        }

        s.push_str(&format!(
            "Feature Schema: v{}\n",
            self.provenance.feature_schema_version
        ));
        s.push_str(&format!(
            "Analysis Time: {}\n",
            chrono::DateTime::from_timestamp(self.provenance.analysis_timestamp, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| "unknown".to_string())
        ));

        if let Some(duration) = self.provenance.analysis_duration_secs {
            s.push_str(&format!("Analysis Duration: {:.2}s\n", duration));
        }

        s.push_str(&format!("ML Enabled: {}\n", self.provenance.ml_enabled));
        s.push_str(&format!(
            "Static Analysis: {}\n",
            self.provenance.static_enabled
        ));

        if !self.provenance.stages_used.is_empty() {
            s.push_str(&format!(
                "Stages: {}\n",
                self.provenance.stages_used.join(" → ")
            ));
        }

        if let Some(model_path) = &self.provenance.model_path {
            s.push_str(&format!("Model: {}\n", model_path));
        }

        s
    }

    pub fn format_explanation(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!("Function: {}\n", self.function_name));
        output.push_str(&format!("Verdict: {:?}\n", self.label));
        output.push_str(&format!("Confidence: {:.1}%\n\n", self.confidence * 100.0));

        // Show raw ML probability if available
        if let Some(dead_prob) = self.dead_probability {
            output.push_str(&format!(
                "ML Probability (Dead): {:.1}%\n",
                dead_prob * 100.0
            ));
        }

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
        output.push_str("\n---\n");
        output.push_str(&self.format_provenance());

        output
    }
}

// Verdict Config and Engine
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
            dead_threshold: 0.80,
            alive_threshold: 0.15,
            enable_ml: true,
            enable_static: true,
            model_version: None,
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

impl VerdictStats {
    pub fn dead_ratio(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.dead as f64 / self.total as f64
        }
    }
}

#[derive(Clone, Debug)]
pub struct VerdictEngine {
    config: VerdictConfig,
    ml_model: Option<DeadCodeClassifier>,
    dynamic_refs: Option<Vec<DynamicReference>>,
    provenance: VerdictProvenance,
}

impl VerdictEngine {
    const ML_WEIGHT: f64 = 0.4;

    pub fn new(config: VerdictConfig) -> Self {
        Self {
            config,
            ml_model: None,
            dynamic_refs: None,
            provenance: VerdictProvenance {
                analysis_version: env!("CARGO_PKG_VERSION").to_string(),
                model_version: None,
                commit_sha: None,
                feature_schema_version: 1,
                analysis_timestamp: chrono::Utc::now().timestamp(),
                analysis_duration_secs: None,
                stages_used: vec![
                    "root_detection".to_string(),
                    "reachability".to_string(),
                    "verdict".to_string(),
                ],
                ml_enabled: false,
                static_enabled: true,
                model_path: None,
            },
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

    pub fn with_dead_threshold(mut self, threshold: f64) -> Self {
        self.config.dead_threshold = threshold;
        self
    }

    pub fn with_alive_threshold(mut self, threshold: f64) -> Self {
        self.config.alive_threshold = threshold;
        self
    }

    pub fn with_model_version(mut self, version: &str) -> Self {
        self.config.model_version = Some(version.to_string());
        self
    }

    fn find_dynamic_ref<'a>(&'a self, func: &FunctionNode) -> Option<&'a DynamicReference> {
        self.dynamic_refs.as_ref()?.iter().find(|r| {
            r.target_full_path.as_deref() == Some(func.full_path.as_str())
                || r.target_function.as_deref() == Some(func.name.as_str())
                || r.target_pattern.contains(&func.name)
        })
    }

    /// Set the Git commit SHA
    pub fn with_commit_sha(mut self, commit_sha: &str) -> Self {
        self.provenance.commit_sha = Some(commit_sha.to_string());
        self
    }

    /// Set the model path
    pub fn with_model_path(mut self, model_path: &str) -> Self {
        self.provenance.model_path = Some(model_path.to_string());
        if let Some(_model) = &self.ml_model {
            self.provenance.model_version = Some(
                self.config
                    .model_version
                    .clone()
                    .unwrap_or_else(|| "unknown".to_string()),
            );
            self.provenance.ml_enabled = true;
        }
        self
    }

    /// Set analysis duration
    pub fn with_analysis_duration(mut self, duration_secs: f64) -> Self {
        self.provenance.analysis_duration_secs = Some(duration_secs);
        self
    }

    /// Add a stage to the provenance
    pub fn with_stage(mut self, stage: &str) -> Self {
        self.provenance.stages_used.push(stage.to_string());
        self
    }

    pub fn evaluate_function(
        &self,
        func: &FunctionNode,
        call_graph: &CallGraph,
        reachability: &ReachabilityMap,
    ) -> Verdict {
        if is_never_dead(func) {
            return Verdict {
                function_name: func.name.clone(),
                full_path: func.full_path.clone(),
                label: TrainingLabel::Alive,
                state: VerdictState::DefinitelyAlive,
                confidence: 1.0,
                dead_probability: None,
                signals: vec![Signal {
                    name: "never_dead_filter".to_string(),
                    value: 1.0,
                    direction: SignalDirection::SupportsAlive,
                    weight: 1.0,
                    explanation: "Matched a category that is never considered dead \
                                  (trait impl, framework hook, entry point, etc.)"
                        .to_string(),
                }],
                ml_probability: None,
                static_score: Some(1.0),
                explanation: "Filtered: never-dead category".to_string(),
                evidence_sources: vec![EvidenceSource::StaticReachability],
                verified: false,
                verified_by: None,
                provenance: self.provenance.clone(),
            };
        }

        let mut signals = Vec::new();
        let mut evidence_sources = Vec::new();
        let mut static_score = 0.0;

        // Look up dynamic-ref match once, reuse for both signals and evidence sources
        let matched_dynamic_ref = self.find_dynamic_ref(func);

        // 1. Collect static signals (only when static analysis is enabled)
        let normalized_static = if self.config.enable_static {
            let static_signals =
                self.collect_static_signals(func, reachability, matched_dynamic_ref);
            for signal in &static_signals {
                if signal.direction == SignalDirection::SupportsDead {
                    static_score += signal.weight;
                } else if signal.direction == SignalDirection::SupportsAlive {
                    static_score -= signal.weight;
                }
            }
            signals.extend(static_signals);

            evidence_sources.push(EvidenceSource::StaticReachability);
            evidence_sources.push(EvidenceSource::CallGraph);

            let total_weight: f64 = signals.iter().map(|s| s.weight).sum();
            if total_weight > 0.0 {
                ((static_score / total_weight + 1.0) / 2.0).clamp(0.0, 1.0)
            } else {
                0.5
            }
        } else {
            0.5
        };

        // 2. ML prediction (if enabled)
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
                // Use calibrated probability if available
                let dead_prob = if model.calibration.is_some() {
                    1.0 - model.predict_alive_calibrated(&example)
                } else {
                    1.0 - model.predict_probability(&example)
                };

                signals.push(Signal {
                    name: "ml_prediction".to_string(),
                    value: dead_prob,
                    direction: if dead_prob > 0.5 {
                        SignalDirection::SupportsDead
                    } else {
                        SignalDirection::SupportsAlive
                    },
                    weight: Self::ML_WEIGHT,
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

        // 3. Dynamic references — reuse the match found at the top of this function
        if matched_dynamic_ref.is_some() {
            evidence_sources.push(EvidenceSource::DynamicRefs);
        }

        // 4. Combined score
        let combined_score = if let Some(ml) = ml_probability {
            normalized_static * (1.0 - Self::ML_WEIGHT) + ml * Self::ML_WEIGHT
        } else {
            normalized_static
        };

        // 5. Determine state from combined_score
        let state = VerdictState::from_score(
            combined_score,
            self.config.dead_threshold,
            self.config.alive_threshold,
        );

        // 6. Determine label from state
        let (label, confidence) = match state {
            VerdictState::DefinitelyDead | VerdictState::ProbablyDead => {
                (TrainingLabel::Dead, combined_score)
            }
            VerdictState::DefinitelyAlive | VerdictState::ProbablyAlive => {
                (TrainingLabel::Alive, combined_score)
            }
            VerdictState::Unknown => (TrainingLabel::Unknown, 0.5),
        };

        let dead_probability = ml_probability;

        // 7. Generate explanation
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
            dead_probability,
            signals,
            ml_probability,
            static_score: Some(normalized_static),
            explanation,
            evidence_sources,
            verified: false,
            verified_by: None,
            provenance: self.provenance.clone(),
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
        matched_dynamic_ref: Option<&DynamicReference>,
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

        // Dynamic references signal — uses the match passed in from evaluate_function
        if let Some(r) = matched_dynamic_ref {
            let explanation = if r.resolved {
                format!(
                    "Dynamically referenced via {:?} (resolved to this function)",
                    r.reference_type
                )
            } else {
                format!(
                    "Dynamically referenced via {:?} (unresolved target: '{}')",
                    r.reference_type, r.target_pattern
                )
            };

            signals.push(Signal {
                name: "dynamic_reference".to_string(),
                value: 1.0,
                direction: SignalDirection::SupportsAlive,
                weight: 0.4,
                explanation,
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

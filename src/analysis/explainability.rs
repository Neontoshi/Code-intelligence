// src/analysis/explainability.rs
//! Explainability module - makes every verdict explainable

use crate::analysis::verdict_source::Verdict;
use crate::graph::call_graph::FunctionNode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerdictExplanation {
    pub function_name: String,
    pub full_path: String,
    pub verdict: String,
    pub confidence: f64,
    pub evidence: Vec<Evidence>,
    pub timeline: EvidenceTimeline,
    pub risk_assessment: RiskAssessment,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub signal_name: String,
    pub value: f64,
    pub weight: f64,
    pub direction: String,
    pub explanation: String,
    pub source: EvidenceSource,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EvidenceSource {
    StaticReachability,
    CallGraph,
    DynamicRefs,
    MLModel,
    GitHistory,
    HumanReview,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceTimeline {
    pub created_at: Option<String>,
    pub last_modified: Option<String>,
    pub last_commit: Option<String>,
    pub commit_count: usize,
    pub author: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub removal_risk: RiskLevel,
    pub complexity_impact: RiskLevel,
    pub dependency_impact: RiskLevel,
    pub overall_risk: RiskLevel,
    pub estimated_effort: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

pub struct ExplainabilityEngine;

impl ExplainabilityEngine {
    pub fn generate_explanation(
        verdict: &Verdict,
        func: &FunctionNode,
        git_info: Option<&crate::analysis::git_analysis::GitInfo>,
    ) -> VerdictExplanation {
        let mut evidence = Vec::new();

        for signal in &verdict.signals {
            evidence.push(Evidence {
                signal_name: signal.name.clone(),
                value: signal.value,
                weight: signal.weight,
                direction: format!("{:?}", signal.direction),
                explanation: signal.explanation.clone(),
                source: EvidenceSource::StaticReachability,
            });
        }

        if let Some(ml_prob) = verdict.ml_probability {
            evidence.push(Evidence {
                signal_name: "ML Prediction".to_string(),
                value: ml_prob,
                weight: 0.4,
                direction: if ml_prob > 0.5 {
                    "Supports Dead"
                } else {
                    "Supports Alive"
                }
                .to_string(),
                explanation: format!("ML model predicts {:.1}% probability", ml_prob * 100.0),
                source: EvidenceSource::MLModel,
            });
        }

        let timeline = if let Some(git) = git_info {
            EvidenceTimeline {
                created_at: git.commits.last().map(|c| c.date.to_rfc3339()),
                last_modified: Some(git.last_modified.to_rfc3339()),
                last_commit: git.commits.first().map(|c| c.hash.clone()),
                commit_count: git.commits.len(),
                author: git.authors.first().cloned(),
            }
        } else {
            EvidenceTimeline {
                created_at: None,
                last_modified: None,
                last_commit: None,
                commit_count: 0,
                author: None,
            }
        };

        let risk_assessment = Self::assess_risk(verdict, func);
        let recommendation = Self::generate_recommendation(verdict, &risk_assessment);

        VerdictExplanation {
            function_name: verdict.function_name.clone(),
            full_path: verdict.full_path.clone(),
            verdict: format!("{:?}", verdict.label),
            confidence: verdict.confidence,
            evidence,
            timeline,
            risk_assessment,
            recommendation,
        }
    }

    fn assess_risk(_verdict: &Verdict, func: &FunctionNode) -> RiskAssessment {
        let removal_risk = if func.fan_in > 0 {
            RiskLevel::Medium
        } else if func.is_public {
            RiskLevel::High
        } else {
            RiskLevel::Low
        };

        let complexity_impact = if func.complexity > 20.0 {
            RiskLevel::High
        } else if func.complexity > 10.0 {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        };

        let dependency_impact = if func.fan_out > 10 {
            RiskLevel::High
        } else if func.fan_out > 5 {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        };

        let overall_risk =
            if removal_risk == RiskLevel::High || complexity_impact == RiskLevel::High {
                RiskLevel::High
            } else if removal_risk == RiskLevel::Medium || complexity_impact == RiskLevel::Medium {
                RiskLevel::Medium
            } else {
                RiskLevel::Low
            };

        RiskAssessment {
            removal_risk,
            complexity_impact,
            dependency_impact,
            overall_risk,
            estimated_effort: Self::estimate_effort(func),
        }
    }

    fn estimate_effort(func: &FunctionNode) -> String {
        let loc = if func.body_end_line > func.body_start_line {
            func.body_end_line - func.body_start_line + 1
        } else {
            1
        };

        if loc > 100 || func.complexity > 20.0 {
            "High (1-2 days)".to_string()
        } else if loc > 50 || func.complexity > 10.0 {
            "Medium (2-4 hours)".to_string()
        } else {
            "Low (30 min - 1 hour)".to_string()
        }
    }

    fn generate_recommendation(verdict: &Verdict, risk: &RiskAssessment) -> String {
        if verdict.is_dead() {
            match risk.overall_risk {
                RiskLevel::Low => "✅ Safe to remove - low risk, low impact".to_string(),
                RiskLevel::Medium => {
                    "⚠️ Consider removing - medium risk, review dependencies".to_string()
                }
                RiskLevel::High => {
                    "⚠️ Proceed with caution - high risk, requires careful review".to_string()
                }
                RiskLevel::Critical => {
                    "❌ Do not remove without team review - critical function".to_string()
                }
            }
        } else if verdict.is_alive() {
            "✅ Function is alive - keep as is".to_string()
        } else {
            "❓ Unknown - investigate manually".to_string()
        }
    }
}

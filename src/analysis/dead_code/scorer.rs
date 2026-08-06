// src/analysis/dead_code/scorer.rs

use crate::analysis::git_analysis::GitInfo;
use crate::graph::call_graph::FunctionNode;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConfidenceLevel {
    Guaranteed,   // 95-100%
    VeryLikely,   // 80-95%
    Probably,     // 60-80%
    Uncertain,    // 40-60%
    Unlikely,     // 20-40%
    VeryUnlikely, // 0-20%
}

#[derive(Debug, Clone)]
pub struct DeadScore {
    pub score: f64, // 0.0 - 1.0
    pub level: ConfidenceLevel,
    pub factors: Vec<ScoreFactor>,
}

#[derive(Debug, Clone)]
pub struct ScoreFactor {
    pub name: String,
    pub weight: f64,
    pub contribution: f64,
    pub explanation: String,
}

pub struct ConfidenceScorer {
    weights: ScoreWeights,
}

#[derive(Debug, Clone)]
pub struct ScoreWeights {
    pub no_callers: f64,        // 40
    pub is_private: f64,        // 20
    pub no_docs: f64,           // 10
    pub no_tests: f64,          // 15
    pub no_exports: f64,        // 20
    pub no_instantiations: f64, // 20
    pub trait_impl: f64,        // -30 (penalty)
    pub macro_generated: f64,   // -40 (penalty)
    pub public_api: f64,        // -20 (penalty)
    pub last_modified: f64,     // +10 if old
    pub complexity: f64,        // +5 if complex
}

impl Default for ScoreWeights {
    fn default() -> Self {
        Self {
            no_callers: 40.0,
            is_private: 20.0,
            no_docs: 10.0,
            no_tests: 15.0,
            no_exports: 20.0,
            no_instantiations: 20.0,
            trait_impl: -30.0,
            macro_generated: -40.0,
            public_api: -20.0,
            last_modified: 10.0,
            complexity: 5.0,
        }
    }
}

impl ConfidenceScorer {
    pub fn new() -> Self {
        Self {
            weights: ScoreWeights::default(),
        }
    }

    pub fn score_function(&self, func: &FunctionNode, git_info: Option<&GitInfo>) -> DeadScore {
        let mut score = 0.0;
        let mut factors = Vec::new();

        // 1. No callers (+40)
        if func.fan_in == 0 {
            score += self.weights.no_callers;
            factors.push(ScoreFactor {
                name: "no_callers".to_string(),
                weight: self.weights.no_callers,
                contribution: self.weights.no_callers,
                explanation: "Function has no callers".to_string(),
            });
        }

        // 2. Private (-20)
        if !func.is_public {
            score += self.weights.is_private;
            factors.push(ScoreFactor {
                name: "is_private".to_string(),
                weight: self.weights.is_private,
                contribution: self.weights.is_private,
                explanation: "Function is private".to_string(),
            });
        }

        // 3. No documentation (-10)
        if func.doc_comment.is_none() {
            score += self.weights.no_docs;
            factors.push(ScoreFactor {
                name: "no_docs".to_string(),
                weight: self.weights.no_docs,
                contribution: self.weights.no_docs,
                explanation: "No documentation comment".to_string(),
            });
        }

        // 4. No tests (-15)
        if !func.name.starts_with("test_") && !func.name.starts_with("bench_") {
            score += self.weights.no_tests;
            factors.push(ScoreFactor {
                name: "no_tests".to_string(),
                weight: self.weights.no_tests,
                contribution: self.weights.no_tests,
                explanation: "No test or benchmark found".to_string(),
            });
        }

        // 5. No exports (-20)
        if !func.is_public && !func.file.contains("lib.rs") && !func.file.contains("mod.rs") {
            score += self.weights.no_exports;
            factors.push(ScoreFactor {
                name: "no_exports".to_string(),
                weight: self.weights.no_exports,
                contribution: self.weights.no_exports,
                explanation: "Not exported from module".to_string(),
            });
        }

        // 6. Trait implementation penalty (+30)
        if Self::is_trait_method(func) {
            score += self.weights.trait_impl;
            factors.push(ScoreFactor {
                name: "trait_impl".to_string(),
                weight: self.weights.trait_impl.abs(),
                contribution: self.weights.trait_impl,
                explanation: "Trait implementation - may be called polymorphically".to_string(),
            });
        }

        // 7. Macro generated penalty (+40)
        if func.doc_comment.is_some() && func.doc_comment.as_ref().unwrap().contains("macro") {
            score += self.weights.macro_generated;
            factors.push(ScoreFactor {
                name: "macro_generated".to_string(),
                weight: self.weights.macro_generated.abs(),
                contribution: self.weights.macro_generated,
                explanation: "Likely macro-generated code".to_string(),
            });
        }

        // 8. Public API penalty (+20)
        if func.is_public && !func.file.contains("src/bin/") {
            score += self.weights.public_api;
            factors.push(ScoreFactor {
                name: "public_api".to_string(),
                weight: self.weights.public_api.abs(),
                contribution: self.weights.public_api,
                explanation: "Public API - may be used externally".to_string(),
            });
        }

        // 9. Git history: old code (+10)
        if let Some(git) = git_info {
            let days_since_modified = (chrono::Utc::now() - git.last_modified).num_days();
            if days_since_modified > 365 {
                score += self.weights.last_modified;
                factors.push(ScoreFactor {
                    name: "last_modified".to_string(),
                    weight: self.weights.last_modified,
                    contribution: self.weights.last_modified,
                    explanation: format!("Last modified {} days ago", days_since_modified),
                });
            }
        }

        // 10. Complexity bonus (+5)
        if func.complexity > 20.0 {
            score += self.weights.complexity;
            factors.push(ScoreFactor {
                name: "complexity".to_string(),
                weight: self.weights.complexity,
                contribution: self.weights.complexity,
                explanation: format!("Complex function (complexity: {:.2})", func.complexity),
            });
        }

        // Normalize to 0-100
        let max_score = 140.0; // Sum of all positive weights
        let min_score = -90.0; // Sum of all negative weights
        let normalized = (score - min_score) / (max_score - min_score);
        let final_score = normalized.min(1.0).max(0.0);

        let level = match final_score {
            0.95..=1.0 => ConfidenceLevel::Guaranteed,
            0.80..=0.95 => ConfidenceLevel::VeryLikely,
            0.60..=0.80 => ConfidenceLevel::Probably,
            0.40..=0.60 => ConfidenceLevel::Uncertain,
            0.20..=0.40 => ConfidenceLevel::Unlikely,
            _ => ConfidenceLevel::VeryUnlikely,
        };

        DeadScore {
            score: final_score,
            level,
            factors,
        }
    }

    fn is_trait_method(func: &FunctionNode) -> bool {
        func.trait_impl.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::call_graph::FunctionNode;

    #[test]
    fn test_score_private_unused_function() {
        let scorer = ConfidenceScorer::new();
        let func = FunctionNode {
            name: "unused_helper".to_string(),
            full_path: "test::unused_helper".to_string(),
            file: "src/test.rs".to_string(),
            line: 10,
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
        };

        let score = scorer.score_function(&func, None);
        assert!(score.score > 0.8);
        assert!(matches!(
            score.level,
            ConfidenceLevel::VeryLikely | ConfidenceLevel::Guaranteed
        ));
    }

    #[test]
    fn test_score_public_api_function() {
        let scorer = ConfidenceScorer::new();
        let func = FunctionNode {
            name: "public_api".to_string(),
            full_path: "test::public_api".to_string(),
            file: "src/lib.rs".to_string(),
            line: 10,
            is_public: true,
            is_async: false,
            params: vec![],
            returns: vec![],
            complexity: 5.0,
            importance_score: 0.0,
            doc_comment: Some("Public API function".to_string()),
            writes_to: vec![],
            reads_from: vec![],
            errors: vec![],
            fan_in: 0,
            fan_out: 5,
            is_cycle: false,
            depth: 0,
            layer: "core".to_string(),
            trait_impl: None,
        };

        let score = scorer.score_function(&func, None);
        assert!(score.score < 0.6);
    }
}

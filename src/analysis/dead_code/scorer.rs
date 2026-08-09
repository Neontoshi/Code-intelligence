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

    /// Check if a function is a React component
    fn is_react_component(func: &FunctionNode) -> bool {
        let is_tsx = func.file.ends_with(".tsx") || func.file.ends_with(".jsx");
        let is_component = func
            .name
            .chars()
            .next()
            .map(|c| c.is_uppercase())
            .unwrap_or(false);
        is_tsx && is_component
    }

    /// Check if a function is a React hook
    fn is_react_hook(func: &FunctionNode) -> bool {
        func.name.starts_with("use") && !func.name.starts_with("useSolanaGiveaway")
    }

    /// Check if a function is a React state setter
    fn is_state_setter(func: &FunctionNode) -> bool {
        func.name.starts_with("set")
            && func
                .name
                .chars()
                .nth(3)
                .map(|c| c.is_uppercase())
                .unwrap_or(false)
    }

    /// Check if a function is exported
    fn is_exported(func: &FunctionNode) -> bool {
        func.file.contains("mod.rs") || func.file.contains("lib.rs")
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

        // 2. Private (+20)
        if !func.is_public {
            score += self.weights.is_private;
            factors.push(ScoreFactor {
                name: "is_private".to_string(),
                weight: self.weights.is_private,
                contribution: self.weights.is_private,
                explanation: "Function is private".to_string(),
            });
        }

        // 3. No documentation (+10)
        if func.doc_comment.is_none() {
            score += self.weights.no_docs;
            factors.push(ScoreFactor {
                name: "no_docs".to_string(),
                weight: self.weights.no_docs,
                contribution: self.weights.no_docs,
                explanation: "No documentation comment".to_string(),
            });
        }

        // 5. No exports (+20)
        if !func.is_public && !func.file.contains("lib.rs") && !func.file.contains("mod.rs") {
            score += self.weights.no_exports;
            factors.push(ScoreFactor {
                name: "no_exports".to_string(),
                weight: self.weights.no_exports,
                contribution: self.weights.no_exports,
                explanation: "Not exported from module".to_string(),
            });
        }

        // 6. Trait implementation penalty (-30)
        if Self::is_trait_method(func) {
            score += self.weights.trait_impl;
            factors.push(ScoreFactor {
                name: "trait_impl".to_string(),
                weight: self.weights.trait_impl.abs(),
                contribution: self.weights.trait_impl,
                explanation: "Trait implementation - may be called polymorphically".to_string(),
            });
        }

        // 7. Macro generated penalty (-40)
        if func.doc_comment.is_some() && func.doc_comment.as_ref().unwrap().contains("macro") {
            score += self.weights.macro_generated;
            factors.push(ScoreFactor {
                name: "macro_generated".to_string(),
                weight: self.weights.macro_generated.abs(),
                contribution: self.weights.macro_generated,
                explanation: "Likely macro-generated code".to_string(),
            });
        }

        // 8. Public API penalty (-20)
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

        // 11. React components are less likely dead (-30)
        if Self::is_react_component(func) {
            score -= 30.0;
            factors.push(ScoreFactor {
                name: "react_component".to_string(),
                weight: 30.0,
                contribution: -30.0,
                explanation: "React component - likely used in JSX".to_string(),
            });
        }

        // 12. React hooks are less likely dead (-25)
        if Self::is_react_hook(func) {
            score -= 25.0;
            factors.push(ScoreFactor {
                name: "react_hook".to_string(),
                weight: 25.0,
                contribution: -25.0,
                explanation: "React hook - likely used in components".to_string(),
            });
        }

        // 13. State setters are less likely dead (-20)
        if Self::is_state_setter(func) {
            score -= 20.0;
            factors.push(ScoreFactor {
                name: "state_setter".to_string(),
                weight: 20.0,
                contribution: -20.0,
                explanation: "React state setter - used in component state".to_string(),
            });
        }

        // 14. Exported functions are less likely dead (-15)
        if Self::is_exported(func) {
            score -= 15.0;
            factors.push(ScoreFactor {
                name: "exported".to_string(),
                weight: 15.0,
                contribution: -15.0,
                explanation: "Function is exported - may be used externally".to_string(),
            });
        }

        // Normalize to 0-100
        let max_score = 125.0; // Sum of all positive weights
        let min_score = -35.0; // Sum of all negative weights
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

    fn create_test_function(
        name: &str,
        fan_in: usize,
        is_public: bool,
        complexity: f64,
    ) -> FunctionNode {
        FunctionNode {
            name: name.to_string(),
            full_path: format!("test::{}", name),
            file: "src/test.rs".to_string(),
            line: 10,
            is_public,
            is_async: false,
            params: vec![],
            returns: vec![],
            complexity,
            importance_score: 0.0,
            doc_comment: None,
            writes_to: vec![],
            reads_from: vec![],
            errors: vec![],
            fan_in,
            fan_out: 0,
            is_cycle: false,
            depth: 0,
            layer: "core".to_string(),
            trait_impl: None,
        }
    }

    #[test]
    fn test_score_private_unused_function() {
        let scorer = ConfidenceScorer::new();
        let func = create_test_function("unused_helper", 0, false, 1.0);
        let score = scorer.score_function(&func, None);

        // Should be high confidence dead (>= 0.75)
        assert!(
            score.score >= 0.75,
            "Expected score >= 0.75, got {}",
            score.score
        );
        assert!(matches!(
            score.level,
            ConfidenceLevel::Probably | ConfidenceLevel::VeryLikely | ConfidenceLevel::Guaranteed
        ));
    }

    #[test]
    fn test_score_public_api_function() {
        let scorer = ConfidenceScorer::new();
        let func = create_test_function("public_api", 0, true, 5.0);
        let score = scorer.score_function(&func, None);

        // Should be lower confidence (not dead)
        assert!(
            score.score < 0.6,
            "Expected score < 0.6, got {}",
            score.score
        );
    }

    #[test]
    fn test_score_function_with_callers() {
        let scorer = ConfidenceScorer::new();
        let func = create_test_function("used_function", 5, false, 3.0);
        let score = scorer.score_function(&func, None);

        // Should be low confidence (alive)
        assert!(
            score.score < 0.5,
            "Expected score < 0.5, got {}",
            score.score
        );
    }

    #[test]
    fn test_score_trait_implementation() {
        let scorer = ConfidenceScorer::new();
        let mut func = create_test_function("fmt", 0, true, 1.0);
        func.trait_impl = Some("Display".to_string());
        let score = scorer.score_function(&func, None);

        // Trait implementations should have lower dead score
        assert!(
            score.score < 0.7,
            "Expected score < 0.7, got {}",
            score.score
        );
    }

    #[test]
    fn test_score_high_complexity_function() {
        let scorer = ConfidenceScorer::new();
        let func = create_test_function("complex_helper", 0, false, 25.0);
        let score = scorer.score_function(&func, None);

        // High complexity should increase importance (lower dead score)
        assert!(
            score.score < 0.9,
            "Expected score < 0.9, got {}",
            score.score
        );
    }
}

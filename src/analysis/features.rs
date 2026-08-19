//! Dedicated feature extraction - compute all features once, use everywhere

use crate::graph::call_graph::FunctionNode;
use crate::optimize::dedup::core::{compute_ast_hash, compute_exact_hash, compute_signature_hash};
use crate::parser::tree_sitter::ParsedFile;
use std::collections::HashMap;

// Function Features

/// All features extracted from a function - computed once, used everywhere
#[derive(Debug, Clone)]
pub struct FunctionFeatures {
    pub full_path: String,
    pub name: String,
    pub file: String,
    pub line: usize,
    pub signature_hash: String,
    pub ast_hash: String,
    pub body_hash: String,
    pub complexity: f64,
    pub cyclomatic_complexity: f64,
    pub nesting_depth: usize,
    pub line_count: usize,
    pub token_count: usize,
    pub param_count: usize,
    pub return_count: usize,
    pub is_public: bool,
    pub is_async: bool,
    pub call_count: usize,
    pub caller_count: usize,
    pub fan_in: usize,
    pub fan_out: usize,
    pub language: String,
    pub layer: String,
    pub feature_vector: Vec<f64>,
    pub normalized_tokens: Vec<String>,
    pub body: Option<String>,
    pub doc_comment: Option<String>,
}

impl FunctionFeatures {
    /// Create features from a function node and source
    pub fn from_function(func: &FunctionNode, source: Option<&str>, language: &str) -> Self {
        let body = source.map(|s| s.to_string());

        // Compute hashes
        let signature_hash = compute_signature_hash(func);
        let ast_hash = if let Some(src) = source {
            compute_ast_hash(func, src)
        } else {
            String::new()
        };
        let body_hash = if let Some(src) = source {
            compute_exact_hash(func, Some(src))
        } else {
            compute_exact_hash(func, None)
        };

        // Compute complexity
        let (complexity, cyclomatic, nesting) = if let Some(src) = source {
            Self::compute_complexity_metrics(src)
        } else {
            (1.0, 1.0, 0)
        };

        // Compute tokens
        let normalized_tokens = if let Some(src) = source {
            Self::normalize_tokens(src)
        } else {
            Vec::new()
        };

        // Build feature vector
        let feature_vector = Self::build_feature_vector(
            &signature_hash,
            &ast_hash,
            func,
            complexity,
            &normalized_tokens,
        );

        // Line count
        let line_count = source.map(|s| s.lines().count()).unwrap_or(1);
        let token_count = source.map(|s| s.split_whitespace().count()).unwrap_or(0);

        Self {
            full_path: func.full_path.clone(),
            name: func.name.clone(),
            file: func.file.clone(),
            line: func.line,

            signature_hash,
            ast_hash,
            body_hash,

            complexity,
            cyclomatic_complexity: cyclomatic,
            nesting_depth: nesting,
            line_count,
            token_count,

            param_count: func.params.len(),
            return_count: func.returns.len(),
            is_public: func.is_public,
            is_async: func.is_async,

            call_count: func.fan_out,
            caller_count: func.fan_in,
            fan_in: func.fan_in,
            fan_out: func.fan_out,

            language: language.to_string(),
            layer: func.layer.clone(),

            feature_vector,
            normalized_tokens,

            body,
            doc_comment: func.doc_comment.clone(),
        }
    }

    // Computation Helpers
    fn compute_complexity_metrics(source: &str) -> (f64, f64, usize) {
        let mut complexity: f64 = 1.0;
        let mut cyclomatic: f64 = 1.0;
        let mut max_nesting: usize = 0;
        let mut current_nesting: usize = 0;

        let control_flow_patterns = [
            ("if", 0.5),
            ("else if", 0.3),
            ("for", 0.5),
            ("while", 0.5),
            ("loop", 0.3),
            ("match", 0.5),
            ("switch", 0.5),
            ("case", 0.2),
            ("&&", 0.2),
            ("||", 0.2),
            ("?", 0.3),
            ("catch", 0.3),
            ("try", 0.2),
            ("unwrap", 0.2),
            ("expect", 0.2),
        ];

        for line in source.lines() {
            let trimmed = line.trim();

            // Track nesting
            if trimmed.contains('{') {
                current_nesting += 1;
                max_nesting = max_nesting.max(current_nesting);
            }
            if trimmed.contains('}') {
                current_nesting = current_nesting.saturating_sub(1);
            }

            // Control flow complexity
            for (pattern, weight) in &control_flow_patterns {
                if trimmed.contains(pattern) {
                    complexity += weight;
                    cyclomatic += 1.0;
                }
            }
        }

        // Nesting penalty
        complexity += max_nesting as f64 * 0.2;

        // Cap at reasonable maximum
        let complexity = complexity.min(50.0);
        let cyclomatic = cyclomatic.min(50.0);

        (complexity, cyclomatic, max_nesting)
    }

    fn normalize_tokens(source: &str) -> Vec<String> {
        use regex::Regex;

        let mut tokens = Vec::new();

        // Replace identifiers with placeholders
        let id_regex = Regex::new(r"\b([a-zA-Z_][a-zA-Z0-9_]*)\b").unwrap();
        let mut var_counter = 0;
        let mut var_map = HashMap::new();

        for word in source.split_whitespace() {
            let word = word.trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '_');

            if word.is_empty() {
                continue;
            }

            // Check if it's an identifier
            if id_regex.is_match(word) {
                let skip_words = [
                    "if", "else", "for", "while", "match", "fn", "pub", "async", "await", "return",
                    "let", "mut", "struct", "enum", "trait", "impl", "use", "mod", "true", "false",
                    "null", "None", "Some", "Ok", "Err", "Result", "Option", "Vec", "String",
                    "Box", "Arc", "Rc", "self", "Self", "super", "crate",
                ];
                if !skip_words.contains(&word) {
                    let var_id = var_map.entry(word.to_string()).or_insert_with(|| {
                        var_counter += 1;
                        var_counter
                    });
                    tokens.push(format!("VAR{}", var_id));
                    continue;
                }
            }

            // Keep as is
            tokens.push(word.to_string());
        }

        tokens
    }

    fn build_feature_vector(
        signature_hash: &str,
        ast_hash: &str,
        func: &FunctionNode,
        complexity: f64,
        tokens: &[String],
    ) -> Vec<f64> {
        let mut features = Vec::new();

        // 1. Complexity
        features.push(complexity / 50.0);

        // 2. Parameter count
        features.push(func.params.len() as f64 / 10.0);

        // 3. Return count
        features.push(func.returns.len() as f64 / 5.0);

        // 4. Public/Async flags
        features.push(if func.is_public { 1.0 } else { 0.0 });
        features.push(if func.is_async { 1.0 } else { 0.0 });

        // 5. Token count
        features.push(tokens.len() as f64 / 100.0);

        // 6. Fan-in/Fan-out
        features.push(func.fan_in as f64 / 10.0);
        features.push(func.fan_out as f64 / 10.0);

        // 7. Hash entropy (simplified)
        features.push(signature_hash.chars().take(8).filter(|&c| c > '7').count() as f64 / 8.0);
        features.push(ast_hash.chars().take(8).filter(|&c| c > '7').count() as f64 / 8.0);

        features
    }

    // Similarity Methods

    /// Cosine similarity between two feature vectors
    pub fn cosine_similarity(&self, other: &FunctionFeatures) -> f64 {
        let a = &self.feature_vector;
        let b = &other.feature_vector;

        if a.is_empty() || b.is_empty() || a.len() != b.len() {
            return 0.0;
        }

        let dot: f64 = a.iter().zip(b).map(|(x, y)| x * y).sum();
        let norm_a: f64 = a.iter().map(|x| x.powi(2)).sum::<f64>().sqrt();
        let norm_b: f64 = b.iter().map(|x| x.powi(2)).sum::<f64>().sqrt();

        if norm_a > 0.0 && norm_b > 0.0 {
            dot / (norm_a * norm_b)
        } else {
            0.0
        }
    }

    /// Token overlap similarity
    pub fn token_overlap(&self, other: &FunctionFeatures) -> f64 {
        let a: std::collections::HashSet<_> = self.normalized_tokens.iter().collect();
        let b: std::collections::HashSet<_> = other.normalized_tokens.iter().collect();

        let intersection = a.intersection(&b).count();
        let union = a.len() + b.len() - intersection;

        if union == 0 {
            1.0
        } else {
            intersection as f64 / union as f64
        }
    }
}

// Feature Extractor

#[derive(Debug)]
pub struct FeatureExtractor {
    features: HashMap<String, FunctionFeatures>,
}

impl FeatureExtractor {
    pub fn new() -> Self {
        Self {
            features: HashMap::new(),
        }
    }
    pub fn insert(&mut self, full_path: String, feature: FunctionFeatures) {
        self.features.insert(full_path, feature);
    }

    /// Extract features for all functions in a codebase
    pub fn extract_all(
        &mut self,
        functions: &[FunctionNode],
        files: &[ParsedFile],
    ) -> &HashMap<String, FunctionFeatures> {
        // Build source map for quick lookup
        let source_map: HashMap<String, &str> = files
            .iter()
            .flat_map(|f| {
                f.functions.iter().map(move |fi| {
                    let full_path = format!("{}::{}", f.path, fi.name);
                    let range = &fi.body_range;
                    let source = &f.source[range.0..range.1];
                    (full_path, source)
                })
            })
            .collect();

        // Build language map
        let lang_map: HashMap<String, &str> = files
            .iter()
            .flat_map(|f| {
                f.functions.iter().map(move |fi| {
                    let full_path = format!("{}::{}", f.path, fi.name);
                    (full_path, f.language.as_str())
                })
            })
            .collect();

        for func in functions {
            let full_path = &func.full_path;
            let source = source_map.get(full_path).copied();
            let language = lang_map.get(full_path).copied().unwrap_or("unknown");

            let features = FunctionFeatures::from_function(func, source, language);
            self.features.insert(full_path.clone(), features);
        }

        &self.features
    }

    /// Get features for a specific function
    pub fn get(&self, full_path: &str) -> Option<&FunctionFeatures> {
        self.features.get(full_path)
    }

    /// Get all features
    pub fn all(&self) -> &HashMap<String, FunctionFeatures> {
        &self.features
    }

    /// Get feature vector for ML
    pub fn get_feature_vector(&self, full_path: &str) -> Option<Vec<f64>> {
        self.features
            .get(full_path)
            .map(|f| f.feature_vector.clone())
    }

    /// Get normalized tokens for a function
    pub fn get_tokens(&self, full_path: &str) -> Option<Vec<String>> {
        self.features
            .get(full_path)
            .map(|f| f.normalized_tokens.clone())
    }
}

impl Default for FeatureExtractor {
    fn default() -> Self {
        Self::new()
    }
}

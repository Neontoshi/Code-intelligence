use crate::optimize::dedup::types::ASTSignature;
use std::collections::HashSet;
use tree_sitter::{Node, Parser};

pub struct ASTAnalyzer {
    parser: Parser,
}

impl ASTAnalyzer {
    pub fn new() -> Self {
        let parser = Parser::new();
        // Would set language based on file type
        Self { parser }
    }

    pub fn analyze(&mut self, source: &str) -> ASTSignature {
        if let Some(tree) = self.parser.parse(source, None) {
            let root = tree.root_node();
            let node_types = self.collect_node_types(root);
            let depth = self.calculate_depth(root);

            ASTSignature {
                node_types,
                depth,
                complexity: self.calculate_complexity(root),
            }
        } else {
            ASTSignature {
                node_types: Vec::new(),
                depth: 0,
                complexity: 0.0,
            }
        }
    }

    pub fn similarity(&self, a: &ASTSignature, b: &ASTSignature) -> f64 {
        let mut score = 0.0;
        let mut total = 0.0;

        // Node type similarity (50%)
        let common = a
            .node_types
            .iter()
            .filter(|t| b.node_types.contains(t))
            .count();
        let union = a.node_types.len() + b.node_types.len() - common;
        if union > 0 {
            score += (common as f64 / union as f64) * 0.5;
        }
        total += 0.5;

        // Depth similarity (25%)
        let depth_diff = (a.depth as i32 - b.depth as i32).abs();
        score += (1.0 - (depth_diff as f64 / 20.0).min(1.0)) * 0.25;
        total += 0.25;

        // Complexity similarity (25%)
        let comp_diff = (a.complexity - b.complexity).abs();
        score += (1.0 - (comp_diff / 10.0).min(1.0)) * 0.25;
        total += 0.25;

        if total > 0.0 {
            score / total
        } else {
            0.0
        }
    }

    fn collect_node_types(&self, node: Node) -> Vec<String> {
        let mut types = HashSet::new();
        self.collect_node_types_recursive(node, &mut types);
        let mut result: Vec<String> = types.into_iter().collect();
        result.sort();
        result
    }

    fn collect_node_types_recursive(&self, node: Node, types: &mut HashSet<String>) {
        let kind = node.kind();
        if ![
            "identifier",
            "string",
            "integer",
            "float",
            "comment",
            "whitespace",
        ]
        .contains(&kind)
        {
            types.insert(kind.to_string());
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.collect_node_types_recursive(child, types);
        }
    }

    fn calculate_depth(&self, node: Node) -> usize {
        let mut max_depth = 0;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            max_depth = max_depth.max(1 + self.calculate_depth(child));
        }
        max_depth
    }

    fn calculate_complexity(&self, node: Node) -> f64 {
        let mut complexity = 1.0;
        let control_flow = ["if", "else", "for", "while", "match", "switch", "case"];

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if control_flow.contains(&child.kind()) {
                complexity += 0.5;
            }
            complexity += self.calculate_complexity(child) * 0.1;
        }

        complexity.min(50.0)
    }
}

impl Default for ASTAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

use crate::graph::call_graph::FunctionNode;

pub struct MLAnalyzer;

impl MLAnalyzer {
    /// Extract feature vector for ML comparison
    pub fn extract_features(func: &FunctionNode, source: &str) -> Vec<f64> {
        let mut features = Vec::new();

        // 1. Size metrics
        features.push((source.split_whitespace().count() as f64) / 1000.0);
        features.push((source.lines().count() as f64) / 100.0);

        // 2. Complexity
        features.push(func.complexity / 50.0);

        // 3. Parameter metrics
        features.push(func.params.len() as f64 / 10.0);
        features.push(func.returns.len() as f64 / 5.0);

        // 4. Async/Public indicators
        features.push(if func.is_async { 1.0 } else { 0.0 });
        features.push(if func.is_public { 1.0 } else { 0.0 });

        // 5. Call metrics
        features.push((func.params.len() + func.returns.len()) as f64 / 10.0);

        // 6. Control flow density
        let control_flow = ["if", "else", "for", "while", "match", "switch"]
            .iter()
            .map(|kw| source.matches(kw).count())
            .sum::<usize>();
        features.push(control_flow as f64 / 20.0);

        // 7. Error handling patterns
        let error_patterns = ["unwrap", "expect", "?", "try!", "catch"]
            .iter()
            .map(|kw| source.matches(kw).count())
            .sum::<usize>();
        features.push(error_patterns as f64 / 10.0);

        // 8. Comment density
        let comment_lines = source
            .lines()
            .filter(|l| l.trim().starts_with("//") || l.trim().starts_with("/*"))
            .count();
        features.push(comment_lines as f64 / 50.0);

        // 9. Type usage
        let type_patterns = ["Vec", "HashMap", "Result", "Option", "String", "Box", "Arc"]
            .iter()
            .map(|t| source.matches(t).count())
            .sum::<usize>();
        features.push(type_patterns as f64 / 10.0);

        // 10. Name length (normalized)
        features.push(func.name.len() as f64 / 50.0);

        features
    }

    pub fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
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

    pub fn euclidean_distance(a: &[f64], b: &[f64]) -> f64 {
        if a.is_empty() || b.is_empty() || a.len() != b.len() {
            return 1.0;
        }

        a.iter()
            .zip(b)
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f64>()
            .sqrt()
    }
}

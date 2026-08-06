//! String utility functions shared across the codebase

/// Calculate Levenshtein distance between two strings
pub fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();

    let mut matrix = vec![vec![0; b_chars.len() + 1]; a_chars.len() + 1];

    for i in 0..=a_chars.len() {
        matrix[i][0] = i;
    }
    for j in 0..=b_chars.len() {
        matrix[0][j] = j;
    }

    for i in 1..=a_chars.len() {
        for j in 1..=b_chars.len() {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };
            matrix[i][j] = std::cmp::min(
                std::cmp::min(matrix[i - 1][j] + 1, matrix[i][j - 1] + 1),
                matrix[i - 1][j - 1] + cost,
            );
        }
    }

    matrix[a_chars.len()][b_chars.len()]
}

/// Calculate similarity ratio between two strings (0.0 - 1.0)
pub fn levenshtein_ratio(a: &str, b: &str) -> f64 {
    let a_lower = a.to_lowercase();
    let b_lower = b.to_lowercase();

    if a_lower == b_lower {
        return 1.0;
    }

    let max_len = a_lower.len().max(b_lower.len());
    if max_len == 0 {
        return 1.0;
    }

    let distance = levenshtein_distance(&a_lower, &b_lower);
    1.0 - (distance as f64 / max_len as f64)
}

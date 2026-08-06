/// Estimates GPT token count for text
pub struct TokenEstimator;

impl TokenEstimator {
    /// Rough estimate: 1 token ≈ 3 characters for code, 4 for English
    pub fn estimate_tokens(text: &str) -> usize {
        let code_chars: usize = text.chars()
            .filter(|c| c.is_ascii_punctuation() || c.is_ascii_digit())
            .count();
        let text_chars: usize = text.chars()
            .filter(|c| c.is_alphabetic() || c.is_whitespace())
            .count();
        
        // Code characters average ~3 chars/token, text ~4 chars/token
        (code_chars / 3) + (text_chars / 4)
    }
    
    /// Compare original vs compressed token counts
    pub fn compare(original: &str, compressed: &str) -> (usize, usize, f64) {
        let orig_tokens = Self::estimate_tokens(original);
        let comp_tokens = Self::estimate_tokens(compressed);
        let reduction = if orig_tokens > 0 {
            (1.0 - comp_tokens as f64 / orig_tokens as f64) * 100.0
        } else {
            0.0
        };
        (orig_tokens, comp_tokens, reduction)
    }
}

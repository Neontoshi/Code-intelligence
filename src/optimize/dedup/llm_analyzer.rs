// src/optimize/dedup/llm_analyzer.rs

use std::process::Command;

pub struct LLMAnalyzer {
    model: String,
    enabled: bool,
}

impl LLMAnalyzer {
    pub fn new() -> Self {
        // Check if Ollama is available
        let enabled = std::process::Command::new("ollama")
            .arg("--version")
            .output()
            .is_ok();

        Self {
            model: "phi:2.7b".to_string(), // Changed to phi:2.7b
            enabled,
        }
    }

    pub fn are_duplicates(&self, source_a: &str, source_b: &str) -> Option<bool> {
        if !self.enabled {
            return None;
        }

        // Simple prompt for phi-2
        let prompt = format!(
            "Answer ONLY yes or no. Are these functions duplicates?\n\nFunction A:\n{}\n\nFunction B:\n{}",
            source_a, source_b
        );

        let output = match Command::new("ollama")
            .args(["run", &self.model, &prompt])
            .output()
        {
            Ok(out) => out,
            Err(_) => return None,
        };

        let response = String::from_utf8_lossy(&output.stdout);
        let response = response.trim().to_lowercase();

        // Check for yes/no
        if response.contains("yes") && !response.contains("no") {
            Some(true)
        } else if response.contains("no") && !response.contains("yes") {
            Some(false)
        } else {
            // First word check
            let first_word = response.split_whitespace().next().unwrap_or("");
            match first_word {
                "yes" | "yeah" | "yep" => Some(true),
                "no" | "nah" | "nope" => Some(false),
                _ => None,
            }
        }
    }

    pub fn is_available(&self) -> bool {
        self.enabled
    }
}

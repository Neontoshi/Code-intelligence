// src/bin/ci/types.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    #[serde(default)]
    pub defaults: Defaults,
    #[serde(default)]
    pub projects: HashMap<String, ProjectConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Defaults {
    pub model: Option<String>,
    pub duplicate_model: Option<String>,
    pub threshold: Option<f64>,
    pub verbose: bool,
    pub llm_provider: Option<String>,
    pub llm_model: Option<String>,
}

impl Default for Defaults {
    fn default() -> Self {
        Self {
            model: None,
            duplicate_model: None,
            threshold: None,
            verbose: false,
            llm_provider: Some("ollama".to_string()),
            llm_model: Some("phi:2.7b".to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub path: String,
    pub project_type: Option<String>,
    pub threshold: Option<f64>,
    pub last_analyzed: Option<String>,
    pub dead_count: Option<usize>,
}

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::analysis::framework_database::load_frameworks;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameworkEntry {
    pub name: String,
    pub language: String,
    pub root_patterns: Vec<String>,
    pub decorator_patterns: Vec<String>,
    pub annotation_patterns: Vec<String>,
    pub registration_patterns: Vec<String>,
    pub generated_entrypoints: Vec<String>,
    pub dynamic_behavior: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct FrameworkRegistry {
    entries: HashMap<String, FrameworkEntry>,
}

impl FrameworkRegistry {
    pub fn new() -> Self {
        let mut registry = Self::default();

        for entry in load_frameworks() {
            registry.register(entry);
        }

        registry
    }

    pub fn register(&mut self, entry: FrameworkEntry) {
        let key = format!("{}_{}", entry.language, entry.name);
        self.entries.insert(key, entry);
    }

    pub fn get(&self, language: &str, name: &str) -> Option<&FrameworkEntry> {
        let key = format!("{}_{}", language, name);
        self.entries.get(&key)
    }

    pub fn get_for_language(&self, language: &str) -> Vec<&FrameworkEntry> {
        self.entries
            .values()
            .filter(|e| e.language == language)
            .collect()
    }

    pub fn is_framework_root(&self, language: &str, source: &str, name: &str) -> bool {
        self.get_for_language(language).iter().any(|entry| {
            entry.root_patterns.iter().any(|p| source.contains(p))
                || entry.decorator_patterns.iter().any(|p| source.contains(p))
                || entry.annotation_patterns.iter().any(|p| source.contains(p))
                || entry
                    .registration_patterns
                    .iter()
                    .any(|p| source.contains(p))
                || entry.generated_entrypoints.iter().any(|p| name.contains(p))
        })
    }

    pub fn is_dynamic_behavior(&self, language: &str, source: &str) -> bool {
        self.get_for_language(language)
            .iter()
            .any(|entry| entry.dynamic_behavior.iter().any(|p| source.contains(p)))
    }
}

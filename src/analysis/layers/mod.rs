// src/analysis/layers/mod.rs

//! Language-specific architectural layer detection
//!
//! Layers are logical groupings of code based on file path conventions.
//! Different languages have different directory structure conventions.

pub mod common;
pub mod cpp;
pub mod csharp;
pub mod dart;
pub mod go;
pub mod java;
pub mod javascript;
pub mod php;
pub mod python;
pub mod rust;
pub mod typescript;

pub use common::*;

use crate::parser::tree_sitter::ParsedFile;
use std::collections::HashMap;

/// A detected architectural layer
#[derive(Debug, Clone)]
pub struct LayerInfo {
    pub name: String,
    pub description: String,
    pub color: String,
}

/// Trait for language-specific layer detectors
pub trait LayerDetector: Send + Sync {
    /// Detect the layer for a file
    fn detect_layer(&self, file: &ParsedFile) -> String;

    /// Get all detected layers with descriptions
    fn get_layer_info(&self) -> HashMap<String, LayerInfo>;
}

/// Main layer detector orchestrator
pub struct LayerOrchestrator {
    detectors: HashMap<String, Box<dyn LayerDetector>>,
}

impl LayerOrchestrator {
    pub fn new() -> Self {
        let mut detectors: HashMap<String, Box<dyn LayerDetector>> = HashMap::new();

        detectors.insert("rust".to_string(), Box::new(rust::RustLayerDetector));
        detectors.insert("python".to_string(), Box::new(python::PythonLayerDetector));
        detectors.insert(
            "javascript".to_string(),
            Box::new(javascript::JavaScriptLayerDetector),
        );
        detectors.insert(
            "typescript".to_string(),
            Box::new(typescript::TypeScriptLayerDetector),
        );
        detectors.insert("go".to_string(), Box::new(go::GoLayerDetector));
        detectors.insert("java".to_string(), Box::new(java::JavaLayerDetector));
        detectors.insert("dart".to_string(), Box::new(dart::DartLayerDetector));
        detectors.insert("php".to_string(), Box::new(php::PhpLayerDetector));
        detectors.insert("cpp".to_string(), Box::new(cpp::CppLayerDetector));
        detectors.insert("csharp".to_string(), Box::new(csharp::CSharpLayerDetector));

        Self { detectors }
    }

    /// Detect the layer for a file
    pub fn detect_layer(&self, file: &ParsedFile) -> String {
        if let Some(detector) = self.detectors.get(&file.language.to_lowercase()) {
            detector.detect_layer(file)
        } else {
            // Fallback to generic detection
            common::detect_layer_generic(file)
        }
    }

    /// Get layer information for a specific language
    pub fn get_layer_info(&self, language: &str) -> HashMap<String, LayerInfo> {
        if let Some(detector) = self.detectors.get(&language.to_lowercase()) {
            detector.get_layer_info()
        } else {
            HashMap::new()
        }
    }

    /// Get all layer info across all languages
    pub fn get_all_layer_info(&self) -> HashMap<String, HashMap<String, LayerInfo>> {
        let mut all_info = HashMap::new();
        for (lang, detector) in &self.detectors {
            all_info.insert(lang.clone(), detector.get_layer_info());
        }
        all_info
    }

    /// Get layer color
    pub fn get_layer_color(&self, layer: &str) -> String {
        common::layer_color(layer)
    }

    /// Get layer description
    pub fn get_layer_description(&self, layer: &str) -> String {
        common::layer_description(layer)
    }
}

impl Default for LayerOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

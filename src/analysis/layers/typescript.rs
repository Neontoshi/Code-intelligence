// src/analysis/layers/typescript.rs

//! TypeScript-specific layer detection

use crate::analysis::layers::{javascript::JavaScriptLayerDetector, LayerDetector, LayerInfo};
use crate::parser::tree_sitter::ParsedFile;
use std::collections::HashMap;

pub struct TypeScriptLayerDetector;

impl LayerDetector for TypeScriptLayerDetector {
    fn detect_layer(&self, file: &ParsedFile) -> String {
        // TypeScript shares most patterns with JavaScript
        let js_detector = JavaScriptLayerDetector;
        let layer = js_detector.detect_layer(file);

        // TypeScript-specific refinements
        if layer == "core" {
            let path = &file.path;
            if path.ends_with(".d.ts") {
                return "types".to_string();
            }
            if path.contains("/interfaces/") || path.contains("/types/") {
                return "types".to_string();
            }
            if path.ends_with(".test.ts") || path.ends_with(".spec.ts") {
                return "test".to_string();
            }
            if path.ends_with(".stories.ts") || path.ends_with(".stories.tsx") {
                return "story".to_string();
            }
        }

        layer
    }

    fn get_layer_info(&self) -> HashMap<String, LayerInfo> {
        let mut info = HashMap::new();

        // Start with JavaScript layers
        let js_info = JavaScriptLayerDetector.get_layer_info();
        info.extend(js_info);

        // Add TypeScript-specific layers
        info.insert(
            "types".to_string(),
            LayerInfo {
                name: "types".to_string(),
                description: "Type definitions and interfaces".to_string(),
                color: "#8b5cf6".to_string(),
            },
        );

        info
    }
}

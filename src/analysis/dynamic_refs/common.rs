// src/analysis/dynamic_refs/common.rs

//! Common types and utilities for dynamic reference detection

use serde::{Deserialize, Serialize};

/// Types of dynamic references
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DynamicRefType {
    Reflection,
    Callback,
    Framework,
    DynamicImport,
    DependencyInjection,
    StringDispatch,
    DynamicDispatch,
    Unknown,
}

/// A dynamic reference detected in the codebase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicReference {
    pub source_file: String,
    pub source_function: Option<String>,
    pub target_function: Option<String>,
    pub target_full_path: Option<String>,
    pub target_pattern: String,
    pub reference_type: DynamicRefType,
    pub confidence: f64,
    pub context: String,
    pub resolved: bool,
}

impl DynamicReference {
    pub fn new_framework(
        source_file: String,
        source_function: Option<String>,
        target_function: String,
        target_full_path: Option<String>,
        target_pattern: String,
        confidence: f64,
        context: String,
    ) -> Self {
        let resolved = target_full_path.is_some();
        Self {
            source_file,
            source_function,
            target_function: Some(target_function),
            target_full_path,
            target_pattern,
            reference_type: DynamicRefType::Framework,
            confidence,
            context,
            resolved,
        }
    }

    pub fn new_dynamic_import(
        source_file: String,
        target_function: String,
        target_full_path: Option<String>,
        target_pattern: String,
        confidence: f64,
        context: String,
    ) -> Self {
        let resolved = target_full_path.is_some();
        Self {
            source_file,
            source_function: None,
            target_function: Some(target_function),
            target_full_path,
            target_pattern,
            reference_type: DynamicRefType::DynamicImport,
            confidence,
            context,
            resolved,
        }
    }

    pub fn new_reflection(
        source_file: String,
        source_function: Option<String>,
        target_function: String,
        target_full_path: Option<String>,
        target_pattern: String,
        confidence: f64,
        context: String,
    ) -> Self {
        let resolved = target_full_path.is_some();
        Self {
            source_file,
            source_function,
            target_function: Some(target_function),
            target_full_path,
            target_pattern,
            reference_type: DynamicRefType::Reflection,
            confidence,
            context,
            resolved,
        }
    }

    pub fn is_resolved(&self) -> bool {
        self.resolved
    }
}

/// Extracted dynamic call from AST
#[derive(Debug, Clone)]
pub struct ExtractedDynamicCall {
    pub enclosing_function: Option<String>,
    pub target_name: String,
    pub pattern: String,
    pub ref_type: DynamicRefType,
    pub confidence: f64,
    pub context: String,
}

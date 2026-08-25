// src/analysis/dynamic_refs/csharp.rs

//! C#-specific dynamic reference detection

use crate::analysis::dynamic_refs::{
    common::DynamicReference, traits::DynamicRefDetector, DetectionContext,
};
use crate::parser::tree_sitter::ParsedFile;

pub struct CSharpDetector;

impl DynamicRefDetector for CSharpDetector {
    fn detect(&self, file: &ParsedFile, _context: &mut DetectionContext) -> Vec<DynamicReference> {
        let mut refs = Vec::new();
        let source = &file.source;

        // Detect reflection
        if source.contains("typeof(") && source.contains(".GetMethod(")
            || source.contains("Activator.CreateInstance")
            || source.contains("MethodInfo.Invoke")
        {
            refs.push(DynamicReference::new_reflection(
                file.path.clone(),
                None,
                "GetMethod".to_string(),
                None,
                "Reflection.Invoke".to_string(),
                0.90,
                "C# System.Reflection invocation".to_string(),
            ));
        }

        // Detect ASP.NET Core attributes
        if source.contains("[HttpGet")
            || source.contains("[HttpPost")
            || source.contains("[HttpPut")
            || source.contains("[HttpDelete")
            || source.contains("[ApiController]")
        {
            refs.push(DynamicReference::new_framework(
                file.path.clone(),
                None,
                "ApiController".to_string(),
                None,
                "[ApiController]".to_string(),
                0.95,
                "ASP.NET Core route action".to_string(),
            ));
        }

        refs
    }
}

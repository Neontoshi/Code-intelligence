use crate::analysis::dynamic_refs::{
    common::{DynamicRefType, DynamicReference},
    traits::DynamicRefDetector,
    DetectionContext,
};
use crate::parser::tree_sitter::ParsedFile;

pub struct CppDetector;

impl DynamicRefDetector for CppDetector {
    fn detect(&self, file: &ParsedFile, context: &mut DetectionContext) -> Vec<DynamicReference> {
        let mut refs = Vec::new();

        // 1. Structural detection: Mark virtual/override class methods as dynamic entry points
        for func in &file.functions {
            let is_virtual_or_override = func
                .decorators
                .iter()
                .any(|d| d == "virtual" || d == "override")
                || func.trait_impl.is_some();

            if is_virtual_or_override {
                refs.push(DynamicReference {
                    source_file: file.path.clone(),
                    source_function: Some(func.name.clone()),
                    target_function: Some(func.name.clone()),
                    target_full_path: None,
                    target_pattern: "virtual method override".to_string(),
                    reference_type: DynamicRefType::DynamicDispatch,
                    confidence: 0.90,
                    context: format!("Virtual or overridden method `{}`", func.name),
                    resolved: true,
                });
            }
        }

        // 2. AST Assignment Scanning: Detect function pointer assignments (e.g. `cb = my_func;`)
        for (assigned_var, func_name) in find_cpp_pointer_assignments(&file.source) {
            if let Some(resolved) = context.resolve_symbol(&func_name) {
                refs.push(DynamicReference {
                    source_file: file.path.clone(),
                    source_function: None,
                    target_function: Some(func_name.clone()),
                    target_full_path: Some(resolved),
                    target_pattern: "pointer assignment".to_string(),
                    reference_type: DynamicRefType::Callback,
                    confidence: 0.85,
                    context: format!(
                        "Function `{}` assigned to pointer `{}`",
                        func_name, assigned_var
                    ),
                    resolved: true,
                });
            }
        }

        refs
    }
}

/// Scans source for function pointer assignments: `field = function_identifier;`
fn find_cpp_pointer_assignments(source: &str) -> Vec<(String, String)> {
    let mut assignments = Vec::new();
    for line in source.lines() {
        if line.contains('=') && !line.contains('(') {
            let parts: Vec<&str> = line.split('=').collect();
            if parts.len() == 2 {
                let lhs = parts[0].trim().trim_start_matches('&');
                let rhs = parts[1]
                    .trim()
                    .trim_end_matches(';')
                    .trim_start_matches('&');
                if !lhs.is_empty() && !rhs.is_empty() && is_identifier(rhs) {
                    assignments.push((lhs.to_string(), rhs.to_string()));
                }
            }
        }
    }
    assignments
}

fn is_identifier(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_alphanumeric() || c == '_')
}

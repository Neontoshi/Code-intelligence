// src/analysis/dynamic_refs/dart.rs

//! Dart-specific dynamic reference detection

use crate::analysis::dynamic_refs::{
    common::{DynamicRefType, DynamicReference},
    traits::DynamicRefDetector,
    DetectionContext,
};
use crate::parser::tree_sitter::ParsedFile;

pub struct DartDetector;

impl DynamicRefDetector for DartDetector {
    fn detect(&self, file: &ParsedFile, context: &mut DetectionContext) -> Vec<DynamicReference> {
        let mut refs = Vec::new();

        // 1. Structural detection: Mark any @override as a dynamic framework entry point
        for func in &file.functions {
            let has_override = func.decorators.iter().any(|d| d.contains("override"))
                || func.trait_impl.is_some()
                || func.is_trait_method;

            if has_override {
                refs.push(DynamicReference::new_framework(
                    file.path.clone(),
                    Some(func.name.clone()),
                    func.container.clone().unwrap_or_default(),
                    None,
                    "FrameworkOverride".to_string(),
                    0.95,
                    format!(
                        "Method `{}` overrides a base interface/class method",
                        func.name
                    ),
                ));
            }
        }

        // 2. Generic tear-off resolution: Match bare identifier callbacks in argument lists
        for tearoff in find_dart_call_arg_identifiers(&file.source) {
            if let Some(resolved_path) = context.resolve_symbol(&tearoff) {
                refs.push(DynamicReference {
                    source_file: file.path.clone(),
                    source_function: None,
                    target_function: Some(tearoff.clone()),
                    target_full_path: Some(resolved_path),
                    target_pattern: "call-argument tear-off".to_string(),
                    reference_type: DynamicRefType::Callback,
                    confidence: 0.85,
                    context: format!("Function `{}` passed as a callback argument", tearoff),
                    resolved: true,
                });
            }
        }

        refs
    }
}

fn find_dart_call_arg_identifiers(content: &str) -> Vec<String> {
    let mut found = Vec::new();
    let bytes = content.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if is_ident_start(bytes[i]) {
            let mut j = i;
            while j < bytes.len() && is_ident_char(bytes[j]) {
                j += 1;
            }

            let mut k = j;
            if k < bytes.len() && bytes[k] == b'<' {
                let mut depth = 1i32;
                k += 1;
                while k < bytes.len() && depth > 0 {
                    match bytes[k] {
                        b'<' => depth += 1,
                        b'>' => depth -= 1,
                        _ => {}
                    }
                    k += 1;
                }
            }

            if k < bytes.len() && bytes[k] == b'(' {
                let args_start = k + 1;
                let mut depth = 1i32;
                let mut m = args_start;
                while m < bytes.len() && depth > 0 {
                    match bytes[m] {
                        b'(' => depth += 1,
                        b')' => depth -= 1,
                        _ => {}
                    }
                    if depth > 0 {
                        m += 1;
                    }
                }
                let args_str = &content[args_start..m.min(content.len())];
                for raw_arg in split_top_level_commas(args_str) {
                    let arg = raw_arg.trim();
                    let value = match arg.find(':') {
                        Some(colon_idx) if !arg[..colon_idx].contains('(') => {
                            arg[colon_idx + 1..].trim()
                        }
                        _ => arg,
                    };
                    if is_plain_identifier(value) && !is_dart_reserved(value) {
                        found.push(value.to_string());
                    }
                }
                i = m;
                continue;
            }
            i = j;
            continue;
        }
        i += 1;
    }

    found
}

fn is_ident_start(b: u8) -> bool {
    b.is_ascii_alphabetic() || b == b'_'
}

fn is_ident_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

fn is_plain_identifier(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .next()
            .map(|c| c.is_alphabetic() || c == '_')
            .unwrap_or(false)
        && s.chars().all(|c| c.is_alphanumeric() || c == '_')
}

fn is_dart_reserved(s: &str) -> bool {
    matches!(
        s,
        "true"
            | "false"
            | "null"
            | "this"
            | "super"
            | "context"
            | "widget"
            | "state"
            | "options"
            | "handler"
            | "error"
    )
}

fn split_top_level_commas(s: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0i32;
    let mut start = 0usize;
    for (idx, ch) in s.char_indices() {
        match ch {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth -= 1,
            ',' if depth == 0 => {
                parts.push(&s[start..idx]);
                start = idx + ch.len_utf8();
            }
            _ => {}
        }
    }
    if start < s.len() {
        parts.push(&s[start..]);
    }
    parts
}

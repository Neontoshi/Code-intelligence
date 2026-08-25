use code_intelligence::{
    optimize::{Deduplicator, SemanticCompressor},
    parser::tree_sitter::TreeSitterParser,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::path::Path;
use tempfile;

/// Generate a large repository for benchmarking
fn generate_large_repo(size: usize) -> tempfile::TempDir {
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path();

    // Generate multiple files with many functions
    for file_idx in 0..(size / 20).max(1) {
        let mut code = String::new();

        // Add imports
        code.push_str("use std::collections::HashMap;\n");
        code.push_str("use std::sync::Arc;\n\n");

        // Generate 20 functions per file
        for func_idx in 0..20 {
            let is_public = func_idx % 3 == 0;
            let is_async = func_idx % 4 == 0;
            let has_params = func_idx % 2 == 0;
            let has_return = func_idx % 3 != 1;
            let has_calls = func_idx % 3 != 2;

            code.push_str(if is_public { "pub " } else { "" });
            code.push_str(if is_async { "async " } else { "" });
            code.push_str(&format!("fn func_{}_{}(", file_idx, func_idx));

            let mut params = Vec::new();
            if has_params {
                params.push("x: i32".to_string());
                if func_idx % 3 == 0 {
                    params.push("y: i32".to_string());
                }
                if func_idx % 5 == 0 {
                    params.push("z: String".to_string());
                }
            }
            code.push_str(&params.join(", "));
            code.push_str(")");

            if has_return {
                code.push_str(" -> i32");
            }
            code.push_str(" {\n");

            // Body
            if has_calls && func_idx > 0 {
                let callee_idx = func_idx - 1;
                if func_idx % 2 == 0 {
                    code.push_str(&format!(
                        "    let result = func_{}_{}(x);\n",
                        file_idx, callee_idx
                    ));
                } else {
                    code.push_str(&format!(
                        "    let result = helper_{}_{}(x);\n",
                        file_idx, callee_idx
                    ));
                }
            }

            // Random complexity
            if func_idx % 5 == 0 {
                code.push_str("    if x > 10 {\n");
                code.push_str("        return x * 2;\n");
                code.push_str("    } else if x > 5 {\n");
                code.push_str("        return x + 10;\n");
                code.push_str("    } else {\n");
                code.push_str("        return x;\n");
                code.push_str("    }\n");
            } else if has_return {
                code.push_str("    x + 1\n");
            } else {
                code.push_str("    println!(\"Hello\");\n");
            }
            code.push_str("}\n\n");

            // Generate some helper functions
            if func_idx % 3 == 0 {
                code.push_str(&format!(
                    "fn helper_{}_{}(x: i32) -> i32 {{\n",
                    file_idx, func_idx
                ));
                code.push_str("    x * 2\n");
                code.push_str("}\n\n");
            }
        }

        // Generate structs and types
        for type_idx in 0..3 {
            code.push_str(&format!("pub struct Type_{} {{\n", type_idx));
            for field_idx in 0..3 {
                let field_type = if field_idx % 2 == 0 { "i32" } else { "String" };
                code.push_str(&format!("    field_{}: {},\n", field_idx, field_type));
            }
            code.push_str("}\n\n");

            code.push_str(&format!("impl Type_{} {{\n", type_idx));
            code.push_str(&format!("    pub fn new() -> Self {{\n"));
            code.push_str(&format!(
                "        Self {{ field_0: 0, field_1: \"\".to_string(), field_2: 0 }}\n"
            ));
            code.push_str("    }\n");
            code.push_str("}\n\n");
        }

        let file_name = format!("file_{:04}.rs", file_idx);
        std::fs::write(path.join(&file_name), code).unwrap();
    }

    // Add a main.rs entry point
    let main_code = r#"
pub fn main() {
    let result = func_0_0(42);
    println!("Result: {}", result);
}
"#;
    std::fs::write(path.join("main.rs"), main_code).unwrap();

    // Add Cargo.toml
    let cargo_toml = r#"[package]
name = "large_repo"
version = "0.1.0"
edition = "2021"
"#;
    std::fs::write(path.join("Cargo.toml"), cargo_toml).unwrap();

    temp_dir
}

fn benchmark_parsing(c: &mut Criterion) {
    let parser = TreeSitterParser::new();
    let file_path = Path::new("src/lib.rs");

    c.bench_function("parse_lib_rs", |b| {
        b.iter(|| {
            let _ = parser.parse_file(black_box(file_path));
        })
    });
}

fn benchmark_dedup(c: &mut Criterion) {
    let parser = TreeSitterParser::new();
    let file_path = Path::new("src/lib.rs");
    let parsed = parser.parse_file(file_path).unwrap();

    // Build a simple call graph for testing
    use code_intelligence::graph::call_graph::{CallGraph, FunctionNode};
    let mut call_graph = CallGraph::new();
    for func in &parsed.functions {
        let node = FunctionNode {
            name: func.name.clone(),
            full_path: format!("{}::{}", file_path.to_str().unwrap_or(""), func.name),
            file: file_path.to_str().unwrap_or("").to_string(),
            line: func.line,
            body_start_line: func.body_start_line,
            body_end_line: func.body_end_line,
            is_public: func.is_public,
            is_async: func.is_async,
            params: func.params.iter().map(|p| p.name.clone()).collect(),
            returns: func.return_type.clone().into_iter().collect(),
            complexity: 1.0,
            importance_score: 0.0,
            doc_comment: func.doc_comment.clone(),
            writes_to: Vec::new(),
            reads_from: Vec::new(),
            errors: Vec::new(),
            fan_in: 0,
            fan_out: 0,
            is_cycle: false,
            depth: 0,
            layer: String::new(),
            trait_impl: func.trait_impl.clone(),
            decorators: Vec::new(),
            is_test: false,
            is_trait_method: false,
            is_trait_default: false,
        };
        call_graph.add_function(node);
    }

    let dedup = Deduplicator::new();
    let files = vec![parsed];

    c.bench_function("dedup_find_duplicates", |b| {
        b.iter(|| {
            let _ = dedup.find_duplicates(black_box(&call_graph), black_box(&files));
        })
    });
}

fn benchmark_compression(c: &mut Criterion) {
    let parser = TreeSitterParser::new();
    let file_path = Path::new("src/lib.rs");
    let parsed = parser.parse_file(file_path).unwrap();

    use code_intelligence::graph::call_graph::{CallGraph, FunctionNode};
    let mut call_graph = CallGraph::new();
    for func in &parsed.functions {
        let node = FunctionNode {
            name: func.name.clone(),
            full_path: format!("{}::{}", file_path.to_str().unwrap_or(""), func.name),
            file: file_path.to_str().unwrap_or("").to_string(),
            line: func.line,
            body_start_line: func.body_start_line,
            body_end_line: func.body_end_line,
            is_public: func.is_public,
            is_async: func.is_async,
            params: func.params.iter().map(|p| p.name.clone()).collect(),
            returns: func.return_type.clone().into_iter().collect(),
            complexity: 1.0,
            importance_score: 0.0,
            doc_comment: func.doc_comment.clone(),
            writes_to: Vec::new(),
            reads_from: Vec::new(),
            errors: Vec::new(),
            fan_in: 0,
            fan_out: 0,
            is_cycle: false,
            depth: 0,
            layer: String::new(),
            trait_impl: func.trait_impl.clone(),
            decorators: Vec::new(),
            is_test: false,
            is_trait_method: false,
            is_trait_default: false,
        };
        call_graph.add_function(node);
    }

    let compressor = SemanticCompressor::new();
    let files = vec![parsed];

    c.bench_function("compression_compress", |b| {
        b.iter(|| {
            let _ = compressor.compress(black_box(&call_graph), black_box(&files));
        })
    });
}

fn benchmark_large_repo_compression(c: &mut Criterion) {
    use code_intelligence::optimize::SemanticCompressor;
    use code_intelligence::Pipeline;

    let temp_dir = generate_large_repo(500);
    let path = temp_dir.path();

    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut pipeline = Pipeline::new();
    let analysis = rt.block_on(pipeline.process_project(path)).unwrap();

    let compressor = SemanticCompressor::new();

    c.bench_function("compression_large_repo", |b| {
        b.iter(|| {
            let result =
                compressor.compress(black_box(&analysis.call_graph), black_box(&analysis.files));
            black_box(result.len())
        })
    });
}

criterion_group!(
    benches,
    benchmark_parsing,
    benchmark_dedup,
    benchmark_compression,
    benchmark_large_repo_compression
);
criterion_main!(benches);

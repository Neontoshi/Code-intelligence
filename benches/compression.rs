use code_intelligence::{
    optimize::{Deduplicator, SemanticCompressor},
    parser::tree_sitter::TreeSitterParser,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::path::Path;

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

criterion_group!(
    benches,
    benchmark_parsing,
    benchmark_dedup,
    benchmark_compression
);
criterion_main!(benches);

// src/bin/ci/graph.rs

use code_intelligence::error::Result;
use code_intelligence::graph::GraphMetrics;
use code_intelligence::{
    output::{InteractiveGraph, OverviewGraph},
    Pipeline,
};
use std::path::{Path, PathBuf};

pub async fn run_graph(path: &Path, output: Option<PathBuf>, mode: &str) -> Result<()> {
    let output_file = output.unwrap_or_else(|| {
        if mode == "overview" {
            PathBuf::from("call_graph_overview.html")
        } else {
            PathBuf::from("call_graph.html")
        }
    });

    println!("📊 Generating {} call graph for: {:?}", mode, path);

    let mut pipeline = Pipeline::new();
    let analysis = pipeline.process_project(path).await?;

    let project_name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let html = if mode == "overview" {
        OverviewGraph::generate(&analysis.call_graph, &project_name)
    } else {
        InteractiveGraph::generate(&analysis.call_graph, &analysis.files, &project_name)
    };

    std::fs::write(&output_file, html)?;

    println!("✅ HTML saved to: {:?}", output_file);
    println!("   Functions: {}", analysis.call_graph.node_count());
    println!("   Edges: {}", analysis.call_graph.edge_count());

    Ok(())
}

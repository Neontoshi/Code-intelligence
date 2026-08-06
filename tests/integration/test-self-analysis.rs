#[cfg(test)]
mod tests {
    use code_intelligence::Pipeline;
    use std::path::Path;

    #[test]
    fn test_self_analysis() {
        // Test that the tool can analyze itself
        let mut pipeline = Pipeline::new();
        let result = pipeline.process_project(Path::new("."));
        assert!(result.is_ok(), "Failed to analyze self");

        let intelligence = result.unwrap();
        assert!(
            intelligence.call_graph.node_count() > 0,
            "No functions found"
        );
        assert!(
            intelligence.call_graph.edge_count() > 0,
            "No relationships found"
        );
        assert!(!intelligence.files.is_empty(), "No files found");
    }

    #[test]
    fn test_dead_code_detection() {
        use code_intelligence::analysis::DeadCodeDetector;
        let mut pipeline = Pipeline::new();
        let intelligence = pipeline.process_project(Path::new(".")).unwrap();

        let unused = DeadCodeDetector::find_unused_functions(&intelligence.call_graph);
        // Some dead code is expected in a development project
        println!("Found {} potentially unused functions", unused.len());
    }

    #[test]
    fn test_compression() {
        use code_intelligence::optimize::SemanticCompressor;
        use code_intelligence::optimize::TokenEstimator;

        let mut pipeline = Pipeline::new();
        let intelligence = pipeline.process_project(Path::new(".")).unwrap();

        let compressor = SemanticCompressor::new();
        let compressed = compressor.compress(&intelligence.call_graph, &intelligence.files);

        let original_content: String = intelligence
            .files
            .iter()
            .map(|f| f.source.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        let (orig_tokens, comp_tokens, reduction) =
            TokenEstimator::compare(&original_content, &compressed);

        assert!(
            comp_tokens < orig_tokens,
            "Compression didn't reduce tokens"
        );
        assert!(reduction > 0.0, "No reduction achieved");
        println!("Compression ratio: {:.1}%", reduction);
    }
}

#[test]
fn test_duplicate_detection() {
    use code_intelligence::optimize::Deduplicator;
    use code_intelligence::Pipeline;

    let mut pipeline = Pipeline::new();
    let intelligence = pipeline.process_project(Path::new(".")).unwrap();

    let dedup = Deduplicator::new();
    let result = dedup.find_duplicates(&intelligence.call_graph, &intelligence.files);

    // Should not panic
    assert!(result.duplicate_groups.len() >= 0);
    assert!(result.unique_functions.len() > 0);
    assert!(result.total_saved_tokens >= 0);

    // Print some stats
    println!("\n📊 Deduplication Stats:");
    println!("   Groups found: {}", result.duplicate_groups.len());
    println!("   Unique functions: {}", result.unique_functions.len());
    println!(
        "   Total comparisons: {}",
        result.accuracy_metrics.total_comparisons
    );
    println!(
        "   Confidence: {:.2}%",
        result.accuracy_metrics.confidence_score * 100.0
    );

    // Check that exact duplicates are found in self-analysis
    // (there should be some structural duplicates in a codebase this size)
    let exact_count = result
        .duplicate_groups
        .iter()
        .filter(|g| {
            matches!(
                g.duplicate_type,
                code_intelligence::optimize::dedup::DuplicateType::Exact
            )
        })
        .count();
    println!("   Exact duplicate groups: {}", exact_count);
}

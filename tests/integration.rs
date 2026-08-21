// tests/integration.rs

//! Integration tests entry point

#[path = "integration/mod.rs"]
mod integration;

#[cfg(test)]
mod pipeline_tests {
    use code_intelligence::graph::GraphMetrics;
    use code_intelligence::Pipeline;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_pipeline_full_analysis() {
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path();

        let code = r#"
pub fn main() {
    let result = helper(42);
    println!("{}", result);
}

fn helper(x: i32) -> i32 {
    x * 2
}

fn unused() -> i32 {
    0
}
"#;

        let file_path = temp_path.join("test.rs");
        std::fs::write(&file_path, code).unwrap();

        let mut pipeline = Pipeline::new();
        let analysis = pipeline.process_project(temp_path).await.unwrap();

        assert_eq!(analysis.call_graph.node_count(), 3);
        assert!(analysis.call_graph.edge_count() >= 1);
    }

    #[tokio::test]
    async fn test_pipeline_with_cache() {
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path();

        let code = r#"
pub fn main() {
    println!("Hello");
}
"#;

        let file_path = temp_path.join("test.rs");
        std::fs::write(&file_path, code).unwrap();

        let cache_dir = temp_path.join(".cache");
        let mut pipeline = Pipeline::new().with_cache_dir(cache_dir);

        // First run - cold cache
        let analysis1 = pipeline.process_project(temp_path).await.unwrap();
        assert_eq!(analysis1.call_graph.node_count(), 1);

        // Second run - should use cache
        let analysis2 = pipeline.process_project(temp_path).await.unwrap();
        assert_eq!(analysis2.call_graph.node_count(), 1);
    }

    #[tokio::test]
    async fn test_pipeline_with_git() {
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path();

        let code = r#"
pub fn main() {
    println!("Hello");
}
"#;

        let file_path = temp_path.join("test.rs");
        std::fs::write(&file_path, code).unwrap();

        // Initialize git repo
        std::process::Command::new("git")
            .current_dir(temp_path)
            .args(["init"])
            .output()
            .unwrap();

        let mut pipeline = Pipeline::new().enable_git();
        let analysis = pipeline.process_project(temp_path).await.unwrap();

        // Should still work even without commits
        assert_eq!(analysis.call_graph.node_count(), 1);
    }
}

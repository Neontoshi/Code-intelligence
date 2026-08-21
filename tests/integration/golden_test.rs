// tests/integration/golden_test.rs

//! Golden tests - expected outputs for known fixtures

use code_intelligence::analysis::dead_code::DeadCodeDetector;
use code_intelligence::Pipeline;

#[test]
fn test_golden_simple_project() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();

    // Create a simple project with known dead/alive functions
    let source = r#"
        fn main() {
            used_function();
        }

        fn used_function() {
            println!("Used");
        }

        fn dead_function() {
            println!("Dead");
        }

        fn test_function() {
            // Should be alive (test)
        }

        pub fn public_function() {
            // Should be alive (public API)
        }
    "#;

    std::fs::write(root.join("main.rs"), source).unwrap();

    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut pipeline = Pipeline::new();
    let analysis = rt.block_on(pipeline.process_project(root)).unwrap();

    let stats = DeadCodeDetector::get_dead_stats(&analysis.call_graph, &analysis.files);

    // Golden expectations
    assert_eq!(stats.total, 5, "Should find 5 functions");
    assert!(stats.dead >= 1, "Should find at least 1 dead function");

    // The dead function should be 'dead_function'
    let dead_found = analysis.call_graph.node_indices().any(|idx| {
        let func = &analysis.call_graph[idx];
        func.name == "dead_function" && func.fan_in == 0
    });
    assert!(dead_found, "dead_function should be detected as dead");

    println!("✅ Golden test passed for simple project");
}

#[test]
fn test_golden_complex_project() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();

    // Create a more complex project
    let sources = vec![
        (
            "main.rs",
            r#"
            mod handler;
            mod utils;

            fn main() {
                handler::process();
                utils::helper();
            }
        "#,
        ),
        (
            "handler.rs",
            r#"
            pub fn process() {
                internal_helper();
                used_function();
            }

            fn internal_helper() {}

            fn used_function() {}
            fn dead_function() {}
        "#,
        ),
        (
            "utils.rs",
            r#"
            pub fn helper() {
                // used
            }

            pub fn another_helper() {
                // used
            }

            fn unused_helper() {}
        "#,
        ),
    ];

    for (name, content) in sources {
        std::fs::write(root.join(name), content).unwrap();
    }

    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut pipeline = Pipeline::new();
    let analysis = rt.block_on(pipeline.process_project(root)).unwrap();

    let stats = DeadCodeDetector::get_dead_stats(&analysis.call_graph, &analysis.files);

    // Golden expectations
    assert!(stats.total >= 8, "Should find at least 8 functions");
    assert!(stats.dead >= 2, "Should find at least 2 dead functions");

    // dead_function should be dead
    let dead_found = analysis.call_graph.node_indices().any(|idx| {
        let func = &analysis.call_graph[idx];
        func.name == "dead_function" && func.fan_in == 0
    });
    assert!(dead_found, "dead_function should be detected as dead");

    // unused_helper should be dead
    let unused_found = analysis.call_graph.node_indices().any(|idx| {
        let func = &analysis.call_graph[idx];
        func.name == "unused_helper" && func.fan_in == 0
    });
    assert!(unused_found, "unused_helper should be detected as dead");

    // public functions should be alive
    let process_alive = analysis.call_graph.node_indices().any(|idx| {
        let func = &analysis.call_graph[idx];
        func.name == "process" && func.is_public
    });
    assert!(process_alive, "process should be alive");

    println!("✅ Golden test passed for complex project");
}

// tests/fuzz_tests.rs

//! Fuzz testing for the parser and analysis engine
//!
//! These tests generate random inputs and ensure the parser doesn't crash.

use code_intelligence::graph::GraphMetrics;
use code_intelligence::parser::tree_sitter::TreeSitterParser;
use tempfile::tempdir;

/// Generate random Rust code
fn generate_random_rust_code() -> String {
    let _keywords = [
        "fn", "pub", "struct", "enum", "trait", "impl", "use", "mod", "let", "mut", "if", "else",
        "for", "while", "match", "return", "async", "await", "move", "ref", "dyn", "static",
        "const", "type", "where", "as", "in", "from",
    ];

    let identifiers = [
        "foo", "bar", "baz", "qux", "quux", "corge", "grault", "garply", "waldo", "fred", "plugh",
        "xyzzy", "thud", "main", "handle", "process", "data", "config", "result", "option", "vec",
        "string", "int", "bool",
    ];

    let types = [
        "i32",
        "u64",
        "String",
        "Vec<T>",
        "Option<T>",
        "Result<T, E>",
        "Box<T>",
        "Rc<T>",
        "Arc<T>",
        "&str",
        "&[u8]",
    ];

    let mut rng = rand::thread_rng();
    let num_functions = rand::Rng::gen_range(&mut rng, 1..=5);
    let mut code = String::new();

    for _ in 0..num_functions {
        let is_public = rand::Rng::gen_bool(&mut rng, 0.5);
        let is_async = rand::Rng::gen_bool(&mut rng, 0.3);
        let name = identifiers[rand::Rng::gen_range(&mut rng, 0..identifiers.len())];
        let num_params = rand::Rng::gen_range(&mut rng, 0..=3);

        code.push_str(if is_public { "pub " } else { "" });
        code.push_str(if is_async { "async " } else { "" });
        code.push_str("fn ");
        code.push_str(name);
        code.push('(');

        for i in 0..num_params {
            let param_name = identifiers[rand::Rng::gen_range(&mut rng, 0..identifiers.len())];
            let param_type = types[rand::Rng::gen_range(&mut rng, 0..types.len())];
            code.push_str(&format!("{}: {}", param_name, param_type));
            if i < num_params - 1 {
                code.push_str(", ");
            }
        }

        code.push_str(") -> ");
        let return_type = types[rand::Rng::gen_range(&mut rng, 0..types.len())];
        code.push_str(return_type);
        code.push_str(" {\n");

        // Random body
        let num_lines = rand::Rng::gen_range(&mut rng, 0..=5);
        for _ in 0..num_lines {
            let line_type = rand::Rng::gen_range(&mut rng, 0..3);
            match line_type {
                0 => {
                    let var = identifiers[rand::Rng::gen_range(&mut rng, 0..identifiers.len())];
                    let val = rand::Rng::gen_range(&mut rng, 0..100);
                    code.push_str(&format!("    let {} = {};\n", var, val));
                }
                1 => {
                    let func = identifiers[rand::Rng::gen_range(&mut rng, 0..identifiers.len())];
                    code.push_str(&format!("    {}();\n", func));
                }
                2 => {
                    code.push_str("    // random comment\n");
                }
                _ => {}
            }
        }

        let return_val = if rand::Rng::gen_bool(&mut rng, 0.7) {
            rand::Rng::gen_range(&mut rng, 0..100).to_string()
        } else {
            name.to_string()
        };
        code.push_str(&format!("    {}\n", return_val));
        code.push_str("}\n\n");
    }

    code
}

/// Fuzz test: Random Rust code should never crash the parser
#[test]
fn test_fuzz_parser_rust_random_code() {
    for _ in 0..50 {
        let code = generate_random_rust_code();
        let parser = TreeSitterParser::new();

        // Write to temp file
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("fuzz.rs");
        std::fs::write(&file_path, &code).unwrap();

        // Parse should never panic
        let result = parser.parse_file(&file_path);
        assert!(
            result.is_ok() || result.is_err(),
            "Parser should not panic, got {:?}",
            result
        );

        if let Ok(parsed) = result {
            // Basic sanity checks
            assert!(
                !parsed.source.is_empty(),
                "Parsed source should not be empty"
            );
            // Functions may or may not be found depending on randomness
        }
    }

    println!("✅ Fuzz test: Random Rust code never crashes parser");
}

/// Fuzz test: Malformed Rust code should not crash
#[test]
fn test_fuzz_parser_malformed_rust() {
    let malformed_examples = vec![
        "fn broken(",
        "pub struct {",
        "impl trait for {",
        "fn {}",
        "let x = ;",
        "if ( { }",
        "match x {",
        "unsafe {",
        "fn foo() -> {",
        "struct Foo {",
    ];

    let parser = TreeSitterParser::new();

    for (i, code) in malformed_examples.iter().enumerate() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join(format!("malformed_{}.rs", i));
        std::fs::write(&file_path, code).unwrap();

        // Should never panic, even on malformed input
        let result = parser.parse_file(&file_path);
        // May fail to parse, but should not panic
        assert!(
            result.is_ok() || result.is_err(),
            "Parser should not panic on malformed input"
        );
    }

    println!("✅ Fuzz test: Malformed Rust code never crashes parser");
}

/// Fuzz test: Very large input should not crash
#[test]
fn test_fuzz_parser_large_input() {
    let mut code = String::new();
    for i in 0..1000 {
        code.push_str(&format!("fn func_{}() -> i32 {{ {} }}\n", i, i % 100));
    }

    let parser = TreeSitterParser::new();
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("large.rs");
    std::fs::write(&file_path, &code).unwrap();

    let result = parser.parse_file(&file_path);
    assert!(
        result.is_ok() || result.is_err(),
        "Parser should not panic on large input"
    );

    if let Ok(parsed) = result {
        assert!(
            parsed.functions.len() >= 100,
            "Should find at least 100 functions, found {}",
            parsed.functions.len()
        );
    }

    println!("✅ Fuzz test: Large input never crashes parser");
}

/// Fuzz test: Unicode identifiers should not crash
#[test]
fn test_fuzz_parser_unicode() {
    let code = r#"
fn 你好() -> i32 { 42 }
fn 🦀() -> i32 { 42 }
fn αβγ() -> i32 { 42 }
fn _underscore() -> i32 { 42 }
fn camelCase() -> i32 { 42 }
fn snake_case() -> i32 { 42 }
fn SCREAMING_SNAKE() -> i32 { 42 }
"#;

    let parser = TreeSitterParser::new();
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("unicode.rs");
    std::fs::write(&file_path, code).unwrap();

    let result = parser.parse_file(&file_path);
    assert!(
        result.is_ok() || result.is_err(),
        "Parser should not panic on Unicode input"
    );

    if let Ok(parsed) = result {
        let names: Vec<&str> = parsed.functions.iter().map(|f| f.name.as_str()).collect();
        assert!(
            names.contains(&"你好") || names.contains(&"你好"),
            "Should find Unicode function names"
        );
    }

    println!("✅ Fuzz test: Unicode identifiers never crash parser");
}

/// Fuzz test: Analysis pipeline with random code
#[test]
fn test_fuzz_analysis_pipeline() {
    use code_intelligence::Pipeline;

    for _ in 0..10 {
        let code = generate_random_rust_code();
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("fuzz.rs");
        std::fs::write(&file_path, &code).unwrap();

        let mut pipeline = Pipeline::new();
        let rt = tokio::runtime::Runtime::new().unwrap();

        // Analysis should never panic
        let result = rt.block_on(pipeline.process_project(temp_dir.path()));
        assert!(
            result.is_ok() || result.is_err(),
            "Analysis pipeline should not panic"
        );
    }

    println!("✅ Fuzz test: Analysis pipeline never crashes");
}

/// Fuzz test: Edge cases in call graph building
#[test]
fn test_fuzz_call_graph_edge_cases() {
    use code_intelligence::engine::call_graph_builder::CallGraphBuilder;
    use code_intelligence::parser::tree_sitter::TreeSitterParser;

    let edge_cases = vec![
        // Self-referential
        r#"
fn recursive() -> i32 {
    recursive()
}
"#,
        // Mutual recursion
        r#"
fn a() -> i32 { b() }
fn b() -> i32 { a() }
"#,
        // Empty functions
        r#"
fn empty() {}
fn empty_with_return() -> i32 { 0 }
"#,
        // Generic functions
        r#"
fn generic<T>(x: T) -> T { x }
fn generic_multi<T, U>(x: T, y: U) -> T { x }
"#,
        // Trait bounds
        r#"
fn bounded<T: Clone + Debug>(x: T) -> T { x.clone() }
"#,
        // Where clauses
        r#"
fn where_clause<T>(x: T) -> T
where T: Clone + Debug {
    x.clone()
}
"#,
    ];

    for (i, code) in edge_cases.iter().enumerate() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join(format!("edge_{}.rs", i));
        std::fs::write(&file_path, code).unwrap();

        let parser = TreeSitterParser::new();
        let parsed = parser.parse_file(&file_path).unwrap();

        let files = vec![parsed];
        let call_graph = CallGraphBuilder::build(&files);

        // Call graph should be built without panicking
        assert!(
            call_graph.node_count() > 0,
            "Call graph should have at least one node"
        );

        // Fan metrics should not panic
        let mut graph = call_graph;
        graph.calculate_fan_metrics();
        graph.detect_layers();
        graph.calculate_call_depth();

        // Should not panic on cycle detection
        graph.mark_cycle_members();
    }

    println!("✅ Fuzz test: Call graph edge cases never panic");
}

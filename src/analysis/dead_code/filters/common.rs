// src/analysis/dead_code/filters/common.rs

//! Common utilities for dead code filters

/// Check if a function name matches common boilerplate names
pub fn is_boilerplate_name(name: &str) -> bool {
    let boilerplate_names = [
        "default",
        "clone",
        "fmt",
        "from",
        "into",
        "try_from",
        "try_into",
        "new",
        "len",
        "is_empty",
        "reset",
        "clear",
        "as_ref",
        "as_mut",
        "drop",
        "to_string",
        "to_json",
        "to_jsonl",
        "to_markdown",
        "to_feature_vector",
        "node_count",
        "edge_count",
        "iter_nodes",
        "iter_edges",
        "file_paths",
        "function_names",
        "file_count",
        "function_count",
        "call_edge_count",
    ];
    boilerplate_names.contains(&name)
}

/// Check if a function is a simple getter/setter
pub fn is_simple_accessor(name: &str) -> bool {
    name.starts_with("get_")
        || name.starts_with("set_")
        || name.starts_with("is_")
        || name.starts_with("has_")
        || name.starts_with("push_")
}

/// Check if a function is a builder method
pub fn is_builder_method(name: &str) -> bool {
    name.starts_with("with_")
}

/// Check if a file is a test file
pub fn is_test_file(file: &str) -> bool {
    file.contains("/test/")
        || file.contains("/tests/")
        || file.ends_with("_test.dart")
        || file.ends_with("_test.go")
        || file.ends_with("_test.rs")
        || file.ends_with("_test.py")
        || file.ends_with(".test.js")
        || file.ends_with(".test.ts")
        || file.ends_with("_test.java")
        || file.ends_with("Test.java")
        || file.ends_with("_test.cs")
}

/// Check if a file is a generated file
pub fn is_generated_file(file: &str) -> bool {
    file.contains(".gen.")
        || file.contains("_gen.")
        || file.contains(".g.")
        || file.contains(".freezed.")
        || file.contains(".gr.")
        || file.contains("/generated/")
        || file.contains("/gen/")
        || file.ends_with(".g.dart")
        || file.ends_with(".freezed.dart")
        || file.ends_with(".gr.dart")
        || file.ends_with(".reflectable.dart")
        || file.ends_with(".part.dart")
        || file.ends_with(".pb.go")
        || file.ends_with("_pb2.py")
        || file.ends_with(".gen.go")
}

/// Check if a file is an example file
pub fn is_example_file(file: &str) -> bool {
    file.contains("/example/") || file.contains("/examples/") || file.ends_with(".example.dart")
}

/// Check if a file is in a generated directory
pub fn is_generated_directory(file: &str) -> bool {
    file.contains("/build/")
        || file.contains("/.dart_tool/")
        || file.contains("/.pub-cache/")
        || file.contains("/node_modules/")
        || file.contains("/target/")
        || file.contains("/.cxx/")
        || file.contains("/CMakeFiles/")
        || file.contains("/android/app/build/")
        || file.contains("/ios/Pods/")
        || file.contains("/ios/DerivedData/")
}

/// Check if a file contains framework code
pub fn is_framework_file(file: &str) -> bool {
    let framework_patterns = [
        ".controller.",
        ".service.",
        ".module.",
        ".guard.",
        ".strategy.",
        ".interceptor.",
        ".pipe.",
        ".filter.",
        ".tsx",
        ".jsx",
        ".vue",
        ".svelte",
        "/components/",
        "/pages/",
        "/hooks/",
        "/composables/",
        "/providers/",
        "/contexts/",
        "/layouts/",
        "/tests/",
        "/test/",
        "/bench/",
        "/benches/",
        "/generated/",
        "/gen/",
        "/proto/",
        "/protobuf/",
        "/admin/",
        "/management/",
        "/migrations/",
        "/serializers/",
        "/permissions/",
        "/throttling/",
        "/traits/",
        "/trait/",
        "/impls/",
        "/derive/",
    ];
    for pattern in framework_patterns {
        if file.contains(pattern) {
            return true;
        }
    }
    false
}

/// Check if a file is an extension/plugin file
pub fn is_extension_file(file: &str) -> bool {
    file.contains("/extensions/")
        || file.contains("/intellij/")
        || file.contains("/vscode/")
        || file.contains("/zed/")
        || file.contains("/plugins/")
        || file.contains("/.vscode/")
        // VSCode extension source files
        || (file.ends_with(".ts") && file.contains("/src/") && file.contains("/extensions/"))
        // IntelliJ plugin source files
        || (file.ends_with(".kt") && file.contains("/src/") && file.contains("/intellij/"))
        // Java plugin source files
        || (file.ends_with(".java") && file.contains("/src/") && file.contains("/intellij/"))
}

/// Check if a file is a template file
pub fn is_template_file(file: &str) -> bool {
    file.contains("/templates/")
        || file.ends_with(".template.dart")
        || file.contains("/bricks/")
        || file.ends_with(".template.ts")
        || file.ends_with(".template.js")
        || file.ends_with(".template.java")
        || file.ends_with(".template.kt")
}

/// Check if a function is a constant (all caps or common constant patterns)
pub fn is_constant_name(name: &str) -> bool {
    // All caps with underscores
    if name.chars().all(|c| c.is_uppercase() || c == '_') {
        return true;
    }
    // Common constant patterns
    let constant_patterns = [
        "DEFAULT_",
        "MAX_",
        "MIN_",
        "FILE_NAME",
        "VERSION",
        "TIMEOUT",
        "RETRY",
        "_FILE",
        "_PATH",
        "_NAME",
        "PUBSPEC",
        "ANALYSIS_OPTIONS",
        "_LOCK_FILE",
        "_VALUE",
        "_COUNT",
        "_DELAY",
        "_MS",
    ];
    constant_patterns.iter().any(|p| name.contains(p))
}

/// Check if a function is a framework callback pattern
pub fn is_framework_callback(name: &str) -> bool {
    let callback_patterns = [
        "_subscribe",
        "_unsubscribe",
        "_dispose",
        "_close",
        "_report",
        "_checkForUpdates",
        "_analyzeContent",
        "_analyzeFile",
        "_merge",
        "_fromJson",
        "_toJson",
        "_cast",
        "_startListening",
        "_updateLatestValue",
        "_maybeStreamIdentical",
        "_buildGenerator",
        "_vars",
        "_getLineOffsets",
        "_computeLineOffsets",
        "_ensureBeforeEndOfLine",
        "_lineIgnores",
        "_ignoresAboveLine",
        "_ignoresAfterLine",
        "_isEndOfLine",
        "_getTokens",
        "_getFieldName",
        "_getReturnType",
        "_traverseRead",
        "_traverseWrite",
        "_traverseJson",
        "_traverseAtomicJson",
        "_traverseComplexJson",
        "_removeSeen",
        "_checkCycle",
        "_toEncodable",
        "_observer",
        "_reportDiagnostics",
        "_analyzeDirectory",
    ];
    callback_patterns.contains(&name)
}

/// Check if a function is a snippet generator
pub fn is_snippet_function(name: &str) -> bool {
    name.ends_with("Snippet")
        || name.ends_with("Snippets")
        || name.contains("snippet")
        || name.ends_with("Snippet")
        || name.contains("Snippet")
}

/// Check if a function is a code generator
pub fn is_generator_function(name: &str) -> bool {
    name.contains("generate")
        || name.contains("Generator")
        || name.contains("generator")
        || name.contains("create")
        || name.contains("build")
        || name.ends_with("Generator")
        || name.ends_with("Factory")
}

/// Check if a function is a command handler
pub fn is_command_function(name: &str) -> bool {
    name.ends_with("Command")
        || name.contains("command")
        || name.contains("Command")
        || name.ends_with("Action")
        || name.contains("Action")
        || name.ends_with("Handler")
        || name.contains("Handler")
}

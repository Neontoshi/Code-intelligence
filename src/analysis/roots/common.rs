// src/analysis/roots/common.rs

//! Common utilities for root detection

use crate::graph::call_graph::{CallGraph, FunctionNode};

/// Check if a function is likely a true entry point
pub fn is_likely_entry_point(func: &FunctionNode, call_graph: &CallGraph) -> bool {
    let idx = call_graph.name_index.get(&func.full_path);
    if let Some(&idx) = idx {
        let callers = call_graph.get_callers(idx);
        if callers.is_empty() {
            if func.file.contains("/bin/")
                || func.file.ends_with("main.rs")
                || func.file.contains("/src/bin/")
            {
                return true;
            }

            if func.is_async {
                return true;
            }

            if let Some(doc) = &func.doc_comment {
                if doc.contains("#[tokio::main]")
                    || doc.contains("#[async_std::main]")
                    || doc.contains("entry point")
                {
                    return true;
                }
            }
        }
    }

    false
}

/// Check if a function name suggests a test
pub fn is_test_function_name(name: &str) -> bool {
    name.starts_with("test_")
        || name.starts_with("Test")
        || name.starts_with("bench_")
        || name.starts_with("Benchmark")
        || name.starts_with("Example")
}

/// Check if a file path suggests a test file
pub fn is_test_file_path(file: &str) -> bool {
    file.contains("/tests/")
        || file.contains("/test/")
        || file.ends_with("_test.rs")
        || file.ends_with("_test.go")
        || file.ends_with("_test.py")
        || file.ends_with("test.js")
        || file.ends_with("test.ts")
        || file.ends_with("_test.dart")
        || file.ends_with("Test.java")
}

/// Check if a file path suggests a benchmark file
pub fn is_bench_file_path(file: &str) -> bool {
    file.contains("/benches/")
        || file.ends_with("_bench.rs")
        || file.ends_with("_bench.go")
        || file.ends_with("bench.js")
        || file.ends_with("bench.ts")
}

/// Check if a file path suggests a generated file
pub fn is_generated_file_path(file: &str) -> bool {
    file.contains(".gen.rs")
        || file.contains("_gen.rs")
        || file.contains(".pb.go")
        || file.contains("_pb2.py")
        || file.contains(".generated.")
        || file.contains("/gen/")
        || file.contains("/generated/")
}

/// Check if a function has FFI attributes
pub fn has_ffi_attributes(func: &FunctionNode) -> bool {
    if let Some(doc) = &func.doc_comment {
        if doc.contains("extern \"C\"")
            || doc.contains("#[no_mangle]")
            || doc.contains("#[export_name]")
            || doc.contains("#[link_name]")
            || doc.contains("JNIEXPORT")
            || doc.contains("Q_INVOKABLE")
            || doc.contains("EMSCRIPTEN_KEEPALIVE")
        {
            return true;
        }
    }

    // Check for FFI naming conventions
    if func.name.starts_with('_') && func.name.contains("c_") {
        return true;
    }

    false
}

/// Check if a file contains framework code
pub fn is_framework_file_path(file: &str) -> bool {
    let framework_patterns = [
        "/handlers/",
        "/controllers/",
        "/routes/",
        "/components/",
        "/pages/",
        "/hooks/",
        "/services/",
        "/repositories/",
        "/middleware/",
        "/decorators/",
        "/providers/",
        "/contexts/",
        "/layouts/",
        "/widgets/",
        "/screens/",
        "/views/",
    ];
    for pattern in framework_patterns {
        if file.contains(pattern) {
            return true;
        }
    }
    false
}

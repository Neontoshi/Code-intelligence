// src/analysis/roots/dart.rs

//! Dart-specific root detection

use crate::analysis::roots::{FunctionId, LanguageRootDetector, RootDetectionConfig};
use crate::graph::call_graph::CallGraph;
use crate::parser::tree_sitter::ParsedFile;
use std::collections::HashSet;

pub struct DartRootDetector;

impl LanguageRootDetector for DartRootDetector {
    fn detect_roots(
        &self,
        call_graph: &CallGraph,
        _files: &[ParsedFile],
        config: &RootDetectionConfig,
    ) -> HashSet<FunctionId> {
        let mut roots = HashSet::new();

        for idx in call_graph.node_indices() {
            let func = &call_graph[idx];

            // ============================================================
            // 1. Entry points
            // ============================================================
            if config.include_exports {
                if func.name == "main" {
                    roots.insert(func.full_path.clone());
                    continue;
                }
            }

            // ============================================================
            // 2. Test functions
            // ============================================================
            if config.include_tests {
                if func.is_test || is_test_function_name(&func.name) {
                    roots.insert(func.full_path.clone());
                    continue;
                }
            }

            // ============================================================
            // 3. Flutter Widgets
            // ============================================================
            if config.include_framework {
                if func.file.ends_with(".dart") {
                    let is_widget = func.file.contains("/widgets/")
                        || func.file.contains("/pages/")
                        || func.file.contains("/screens/")
                        || func.file.contains("/views/");

                    if is_widget && func.is_public {
                        roots.insert(func.full_path.clone());
                        continue;
                    }

                    // Flutter lifecycle methods
                    let flutter_lifecycle = [
                        "build",
                        "initState",
                        "dispose",
                        "didUpdateWidget",
                        "didChangeDependencies",
                        "createState",
                        "reassemble",
                        "deactivate",
                        "didChangeAppLifecycleState",
                        "didPushRoute",
                        "didPopRoute",
                        "didHaveMemoryPressure",
                        "didChangeMetrics",
                        "didChangePlatformBrightness",
                        "didChangeTextScaleFactor",
                        "didChangeLocales",
                        "didChangeAccessibilityFeatures",
                    ];
                    if flutter_lifecycle.contains(&func.name.as_str()) {
                        roots.insert(func.full_path.clone());
                        continue;
                    }
                }
            }

            // ============================================================
            // 4. Extension/Plugin entry points
            // ============================================================
            if func.file.contains("/extensions/")
                || func.file.contains("/intellij/")
                || func.file.contains("/vscode/")
                || func.file.contains("/zed/")
            {
                // All exported functions in extensions are entry points
                if func.is_public || !func.name.starts_with('_') {
                    roots.insert(func.full_path.clone());
                    continue;
                }
            }

            // ============================================================
            // 5. VSCode extension specific
            // ============================================================
            if func.file.contains("/vscode/") {
                // All functions in VSCode extension are entry points
                roots.insert(func.full_path.clone());
                continue;
            }

            // ============================================================
            // 6. IntelliJ plugin specific
            // ============================================================
            if func.file.contains("/intellij/") {
                // All functions in IntelliJ plugin are entry points
                roots.insert(func.full_path.clone());
                continue;
            }

            // ============================================================
            // 7. Zed extension specific
            // ============================================================
            if func.file.contains("/zed/") {
                // All functions in Zed extension are entry points
                roots.insert(func.full_path.clone());
                continue;
            }

            // ============================================================
            // 8. Bloc/Cubit generators
            // ============================================================
            if func.file.contains("/generator/")
                || func.file.contains("/bricks/")
                || func.file.contains("/templates/")
            {
                if func.is_public || func.name.contains("generate") {
                    roots.insert(func.full_path.clone());
                    continue;
                }
            }

            // ============================================================
            // 9. Command handlers in bloc_tools
            // ============================================================
            if func.file.contains("/bloc_tools/")
                && func.file.contains("/commands/")
                && (func.name.contains("Command") || func.name.contains("command"))
            {
                roots.insert(func.full_path.clone());
                continue;
            }

            // ============================================================
            // 10. Language server handlers
            // ============================================================
            if func.file.contains("/language_server/") || func.file.contains("/lsp/") {
                if func.is_public || !func.name.starts_with('_') {
                    roots.insert(func.full_path.clone());
                    continue;
                }
            }

            // ============================================================
            // 11. IntelliJ plugin actions
            // ============================================================
            if func.file.contains("/intellij/")
                && (func.name.contains("Action")
                    || func.name.contains("Generator")
                    || func.name.contains("Intention"))
            {
                roots.insert(func.full_path.clone());
                continue;
            }

            // ============================================================
            // 12. VSCode extension commands
            // ============================================================
            if func.file.contains("/vscode/")
                && (func.file.contains("/commands/") || func.file.ends_with(".command.ts"))
            {
                roots.insert(func.full_path.clone());
                continue;
            }

            // ============================================================
            // 13. Public API in packages
            // ============================================================
            if func.file.contains("/packages/") && func.is_public {
                if func.file.contains("/lib/") {
                    roots.insert(func.full_path.clone());
                    continue;
                }
            }

            // ============================================================
            // 14. Framework callback patterns
            // ============================================================
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
            if callback_patterns.contains(&func.name.as_str()) {
                roots.insert(func.full_path.clone());
                continue;
            }

            // ============================================================
            // 15. Snippet/Generator functions
            // ============================================================
            if func.name.ends_with("Snippet")
                || func.name.contains("snippet")
                || func.name.contains("generate")
                || func.name.contains("Generator")
            {
                roots.insert(func.full_path.clone());
                continue;
            }

            // ============================================================
            // 16. Constants
            // ============================================================
            if func.name.chars().all(|c| c.is_uppercase() || c == '_') {
                roots.insert(func.full_path.clone());
                continue;
            }
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
            ];
            if constant_patterns.iter().any(|p| func.name.contains(p)) {
                roots.insert(func.full_path.clone());
                continue;
            }

            // ============================================================
            // 17. Bloc/Cubit generated code
            // ============================================================
            if func.name.contains("Bloc") || func.name.contains("Cubit") {
                if func.name.ends_with("State")
                    || func.name.ends_with("Event")
                    || func.name.ends_with("Bloc")
                    || func.name.ends_with("Cubit")
                {
                    roots.insert(func.full_path.clone());
                    continue;
                }
            }
        }

        roots
    }
}

/// Helper function to check if a name is a test function
fn is_test_function_name(name: &str) -> bool {
    name.starts_with("test_")
        || name.starts_with("Test")
        || name.starts_with("bench_")
        || name.starts_with("Benchmark")
        || name.starts_with("Example")
}

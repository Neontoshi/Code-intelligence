// src/analysis/dead_code/filters/dart.rs

//! Dart-specific dead code filters

use super::common::{
    is_command_function, is_constant_name, is_example_file, is_extension_file,
    is_framework_callback, is_generated_directory, is_generated_file, is_generator_function,
    is_snippet_function, is_template_file, is_test_file,
};
use super::{LanguageFilter, ProtectionLevel};
use crate::graph::call_graph::FunctionNode;

pub struct DartFilter;

impl LanguageFilter for DartFilter {
    fn get_protection_level(&self, func: &FunctionNode) -> ProtectionLevel {
        // ============================================================
        // 1. PROTECTED - Never considered dead
        // ============================================================
        if func.is_override() || func.decorators.iter().any(|d| d.contains("override")) {
            return ProtectionLevel::Protected;
        }

        // Test functions are protected
        if func.is_test || is_test_file(&func.file) {
            return ProtectionLevel::Protected;
        }

        // Generated files are protected
        if is_generated_file(&func.file) || is_generated_directory(&func.file) {
            return ProtectionLevel::Protected;
        }

        // Example files are protected
        if is_example_file(&func.file) {
            return ProtectionLevel::Protected;
        }

        // Flutter Widget lifecycle methods
        if func.file.ends_with(".dart") {
            let flutter_lifecycle = [
                "build",
                "initState",
                "dispose",
                "didUpdateWidget",
                "didChangeDependencies",
                "setState",
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
                return ProtectionLevel::Protected;
            }
        }

        // ============================================================
        // 2. LIKELY ALIVE - Extension/Plugin/Tooling code
        // ============================================================

        // VSCode extension specific - all functions are likely alive
        if func.file.contains("/vscode/") {
            return ProtectionLevel::LikelyAlive;
        }

        // IntelliJ plugin specific - all functions are likely alive
        if func.file.contains("/intellij/") {
            return ProtectionLevel::LikelyAlive;
        }

        // Zed extension specific - all functions are likely alive
        if func.file.contains("/zed/") {
            return ProtectionLevel::LikelyAlive;
        }

        // Extension/Plugin code - called by IDE frameworks
        if is_extension_file(&func.file) {
            // All exported functions in extensions are entry points
            if func.is_public || !func.name.starts_with('_') {
                return ProtectionLevel::LikelyAlive;
            }
        }

        // Template/Generator code - used by code generators
        if is_template_file(&func.file) {
            return ProtectionLevel::LikelyAlive;
        }

        // Constants used in code generation
        if is_constant_name(&func.name) {
            return ProtectionLevel::LikelyAlive;
        }

        // Framework callback patterns
        if is_framework_callback(&func.name) {
            return ProtectionLevel::LikelyAlive;
        }

        // Snippet functions
        if is_snippet_function(&func.name) {
            return ProtectionLevel::LikelyAlive;
        }

        // Generator functions
        if is_generator_function(&func.name) {
            return ProtectionLevel::LikelyAlive;
        }

        // Command/Action handlers
        if is_command_function(&func.name) {
            return ProtectionLevel::LikelyAlive;
        }

        // Private methods in framework packages
        if func.file.contains("/packages/bloc/")
            || func.file.contains("/packages/flutter_bloc/")
            || func.file.contains("/packages/angular_bloc/")
            || func.file.contains("/packages/hydrated_bloc/")
            || func.file.contains("/packages/replay_bloc/")
            || func.file.contains("/packages/bloc_test/")
            || func.file.contains("/packages/bloc_concurrency/")
            || func.file.contains("/packages/bloc_lint/")
        {
            // Private methods in framework packages are called by the framework
            if func.name.starts_with('_') {
                return ProtectionLevel::LikelyAlive;
            }
        }

        // Build/Generator methods in tooling
        if func.file.contains("/bloc_tools/")
            && (func.name.starts_with("_") || func.name.contains("generate"))
        {
            return ProtectionLevel::LikelyAlive;
        }

        // Event handlers in Bloc/Cubit
        if func.name.starts_with("on<")
            || func.name.contains("EventHandler")
            || func.file.contains("/bloc/") && func.name.starts_with("_on")
        {
            return ProtectionLevel::LikelyAlive;
        }

        // IntelliJ plugin actions
        if func.file.contains("/intellij/")
            && (func.name.contains("Action")
                || func.name.contains("Generator")
                || func.name.contains("Intention"))
        {
            return ProtectionLevel::LikelyAlive;
        }

        // VSCode extension commands
        if func.file.contains("/vscode/")
            && (func.file.contains("/commands/") || func.file.ends_with(".command.ts"))
        {
            return ProtectionLevel::LikelyAlive;
        }

        // Language server handlers
        if func.file.contains("/language_server/") || func.file.contains("/lsp/") {
            if func.is_public || !func.name.starts_with('_') {
                return ProtectionLevel::LikelyAlive;
            }
        }

        // Public API in packages
        if func.file.contains("/packages/") && func.is_public {
            if func.file.contains("/lib/") {
                return ProtectionLevel::LikelyAlive;
            }
        }

        // Event handlers - likely used by the framework
        if func.name.starts_with("on") && func.file.contains("/bloc/") {
            return ProtectionLevel::LikelyAlive;
        }

        // ============================================================
        // 3. CANDIDATE - May be dead, needs analysis
        // ============================================================

        // Functions with callers are likely alive
        if func.fan_in > 0 {
            return ProtectionLevel::LikelyAlive;
        }

        // Default: candidate for dead code analysis
        ProtectionLevel::Candidate
    }
}

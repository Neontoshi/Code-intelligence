// src/engine/file_collector.rs

//! File collection with intelligent filtering for source code analysis
//!
//! This module walks directory trees and collects source files while
//! automatically excluding build artifacts, generated code, and other
//! non-source files.

use crate::engine::config::PipelineConfig;
use crate::engine::stages::RawProject;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

// Use lazy_static for compiled regex patterns
use once_cell::sync::Lazy;
use regex::Regex;

/// Patterns for hashed/bundled JavaScript and CSS files
static HASHED_FILE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[a-zA-Z0-9_-]+-[A-Za-z0-9]{6,10}\.(js|css|map)$").unwrap());

/// Patterns for minified files
static MINIFIED_FILE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\.min\.(js|css)$").unwrap());

/// Patterns for bundled files with hash
static BUNDLED_FILE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"-[A-Za-z0-9]{8,}\.(js|css)$").unwrap());

/// Patterns for generated Dart files
static GENERATED_DART_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\.(g|freezed|gr|reflectable|part)\.dart$").unwrap());

/// Patterns for generated Protocol Buffer files
static PROTOBUF_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\.(pb|pbs|pbrpc)\.(go|dart|rs|py)$").unwrap());

/// Patterns for template files
static TEMPLATE_FILE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\.(template|tpl)\.(dart|ts|js|rs)$").unwrap());

pub struct FileCollector;

impl FileCollector {
    pub fn collect(root: &Path, config: &PipelineConfig) -> RawProject {
        // DIRECTORY EXCLUSIONS
        let skip_dirs: &[&str] = &[
            // Version Control
            ".git",
            ".svn",
            ".hg",
            ".bzr",
            // Build Artifacts - General
            "build",
            "target",
            "dist",
            "out",
            "bin",
            "obj",
            ".output",
            // C/C++ / CMake Build
            "cmake-build-debug",
            "cmake-build-release",
            ".cxx", // CMake C++ build artifacts (Flutter NDK)
            "CMakeFiles",
            "CMakeScripts",
            "Testing",
            "_deps",
            // Flutter / Dart
            ".dart_tool",
            ".pub",
            ".pub-cache",
            "build", // Already included but important for Flutter
            ".flutter-plugins",
            ".flutter-plugins-dependencies",
            // Android / Gradle
            ".gradle",
            "gradle",
            "build",
            "libs",
            "generated",
            "intermediates",
            "outputs",
            "tmp",
            // iOS / macOS
            "Pods",
            "DerivedData",
            ".symlinks",
            "xcuserdata",
            ".xcode",
            "build", // iOS build output
            // Node.js / JavaScript
            "node_modules",
            "bower_components",
            ".yarn",
            ".pnp",
            ".pnpm-store",
            // Python
            "__pycache__",
            ".venv",
            "venv",
            ".mypy_cache",
            ".pytest_cache",
            ".tox",
            ".nox",
            ".coverage",
            // Java / JVM
            ".mvn",
            "mvnw",
            "target", // Maven build output
            // Go
            "vendor",
            ".mod",
            ".sum",
            // Rust
            "target", // Cargo build output
            // IDE
            ".idea",
            ".vscode",
            ".vs",
            "nbproject",
            "eclipse",
            ".settings",
            // Testing
            "coverage",
            ".nyc_output",
            ".jest-cache",
            ".cache",
            // Documentation
            "docs",
            "doc",
            "apidoc",
            "gh-pages",
            // Assets / Resources
            "assets",
            "resources",
            "static",
            "public",
            "images",
            "fonts",
            "videos",
            "audio",
            // Remote / External
            "remote-dist",
            "vendor",
            "third_party",
            "third-party",
            // Legacy / Old
            "legacy",
            "old",
            "backup",
            "archive",
            // Test fixtures (large binary/test data)
            "fixtures",
            "testdata",
            "test_data",
            "samples",
            "examples",
        ];

        // SUPPORTED SOURCE CODE EXTENSIONS (ONLY THESE WILL BE PARSED)
        let source_extensions: &[&str] = &[
            // Systems
            "rs", // Rust
            // Web
            "js", "jsx", // JavaScript
            "ts", "tsx", // TypeScript
            // Mobile
            "dart", // Dart/Flutter
            // Backend
            "py",  // Python
            "rb",  // Ruby
            "php", // PHP
            "cs",  // C#
            "lua", // Lua
            // Systems (C family)
            "c", "h", // C
            "cpp", "cc", "cxx", "c++", // C++
            "hpp", "hh", "hxx", "h++", // C++ Headers
            // Other source-like
            "proto",  // Protocol Buffers
            "thrift", // Thrift
            "sql",    // SQL
        ];

        // FILE EXCLUSIONS (FILENAMES AND PATTERNS)
        let skip_files: &[&str] = &[
            // Lock Files
            "package-lock.json",
            "yarn.lock",
            "pnpm-lock.yaml",
            "bun.lockb",
            "bun.lock",
            "npm-shrinkwrap.json",
            "Cargo.lock",
            "poetry.lock",
            "Pipfile.lock",
            "pdm.lock",
            "composer.lock",
            "pubspec.lock",
            "go.sum",
            "Gemfile.lock",
            // Build Metadata
            "gradle-wrapper.jar",
            "gradle-wrapper.properties",
            "pom.xml.tag",
            "pom.xml.releaseBackup",
            "pom.xml.next",
            "pom.xml.backup",
            "packages.lock.json",
            "nuget.config",
            "CMakeCache.txt",
            "compile_commands.json",
            "CMakeUserPresets.json",
            "cmake_install.cmake",
            "CTestTestfile.cmake",
            // CMake Generated Files
            "CMakeCCompilerId.cpp",
            "CMakeCXXCompilerId.cpp",
            "CMakeDetermineCompilerABI_C.bin",
            "CMakeDetermineCompilerABI_CXX.bin",
            "CMakeFiles",
            // Generated Source Files - Dart
            ".g.dart",
            ".freezed.dart",
            ".gr.dart",
            ".reflectable.dart",
            ".part.dart",
            ".gql.dart",
            ".graphql.dart",
            // Generated Source Files - TypeScript/JavaScript
            ".g.ts",
            ".g.js",
            ".g.cs",
            ".d.ts",
            ".d.ts.map",
            ".js.map",
            ".min.js",
            ".min.css",
            ".chunk.js",
            ".chunk.css",
            // Generated Source Files - Protocol Buffers
            ".pb.go",
            ".pb.dart",
            ".pb.rs",
            "_pb2.py",
            "_pb2_grpc.py",
            // Generated Source Files - Rust
            ".rs.bk",
            ".rlib",
            ".rmeta",
            // Generated Source Files - Python
            ".pyc",
            ".pyo",
            ".pyd",
            ".so",
            ".egg-info",
            // Generated Source Files - General
            ".gen",
            ".generated",
            ".auto",
            ".autogen",
            ".mock",
            "_mock",
            "_gen.go",
            ".gen.go",
            ".gen.dart",
            "_gen.dart",
            // IDE Files
            ".DS_Store",
            "Thumbs.db",
            "desktop.ini",
            ".directory",
            // Logs
            ".log",
            ".out",
            ".err",
            "nohup.out",
            // Kotlin files (no tree-sitter grammar installed)
            ".kt",
            ".kts",
            // Swift files (no tree-sitter grammar installed)
            ".swift",
            // Objective-C files (no tree-sitter grammar installed)
            ".m",
            ".mm",
            // Gradle build files
            "build.gradle.kts",
            "settings.gradle.kts",
            "build.gradle",
            "settings.gradle",
            // Android manifest
            "AndroidManifest.xml",
            // iOS/MacOS asset catalogs
            ".xcassets",
            "Contents.json",
            "Info.plist",
            // Shell scripts (not source code for analysis)
            ".sh",
            ".bash",
            ".zsh",
            // TOML config files (not source code for analysis)
            ".toml",
            // Go files (we have the grammar but it's optional)
            // Keep .go since we have tree-sitter-go
        ];

        // COLLECTION
        let files: Vec<PathBuf> = WalkDir::new(root)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_str().unwrap_or("");
                // Skip directories in the skip list
                if skip_dirs.contains(&name) {
                    return false;
                }

                // Skip files that look like generated/bundled assets
                if let Some(name) = e.file_name().to_str() {
                    if HASHED_FILE_RE.is_match(name)
                        || MINIFIED_FILE_RE.is_match(name)
                        || BUNDLED_FILE_RE.is_match(name)
                        || GENERATED_DART_RE.is_match(name)
                        || PROTOBUF_RE.is_match(name)
                        || TEMPLATE_FILE_RE.is_match(name)
                    {
                        return false;
                    }
                    // Skip long names with hash patterns (common in build outputs)
                    if name.len() > 40 && (name.ends_with(".js") || name.ends_with(".css")) {
                        return false;
                    }
                }

                true
            })
            .filter_map(|e| e.ok())
            .filter(|e| {
                if !e.path().is_file() {
                    return false;
                }

                let path_str = e.path().to_string_lossy();

                    // PATH-BASED EXCLUSIONS

                // Skip asset/build directories
                if path_str.contains("/remote-dist/assets/")
                    || path_str.contains("/dist/assets/")
                    || path_str.contains("/build/assets/")
                {
                    return false;
                }

                // Skip CMake build directories (Flutter/Android NDK)
                if path_str.contains("/build/.cxx/")
                    || path_str.contains("/build/intermediates/")
                    || path_str.contains("/.cxx/")
                    || path_str.contains("/CMakeFiles/")
                    || path_str.contains("/CMakeScripts/")
                {
                    return false;
                }

                // Skip Flutter generated files
                if path_str.contains("/.dart_tool/")
                    || path_str.contains("/.pub-cache/")
                    || path_str.contains("/build/generated/")
                    || path_str.contains("/build/tmp/")
                    || path_str.contains("/android/app/build/")
                    || path_str.contains("/ios/Pods/")
                    || path_str.contains("/ios/DerivedData/")
                    // Skip Flutter build directories
                    || path_str.contains("/android/app/src/main/gen/")
                    || path_str.contains("/android/app/src/main/assets/")
                    // Skip Flutter generated plugin registrant
                    || path_str.contains("/android/app/src/main/java/io/flutter/plugins/GeneratedPluginRegistrant.java")
                    // Skip Flutter generated main.dart in build
                    || path_str.contains("/build/flutter_assets/")
                    || path_str.contains("/build/app/")
                {
                    return false;
                }

                // Skip Gradle build directories
                if path_str.contains("/.gradle/")
                    || path_str.contains("/build/generated/")
                    || path_str.contains("/build/tmp/")
                    || path_str.contains("/build/intermediates/")
                    || path_str.contains("/build/outputs/")
                    // Skip Gradle wrapper
                    || path_str.contains("/gradle/wrapper/")
                {
                    return false;
                }

                // Skip Cargo target directory
                if path_str.contains("/target/") {
                    return false;
                }

                // Skip VSCode extension build artifacts
                if path_str.contains("/extensions/vscode/out/")
                    || path_str.contains("/extensions/vscode/node_modules/")
                    || path_str.contains("/extensions/vscode/dist/")
                {
                    return false;
                }

                // Skip IntelliJ plugin build artifacts
                if path_str.contains("/extensions/intellij/build/")
                    || path_str.contains("/extensions/intellij/out/")
                {
                    return false;
                }

                    // FILENAME-BASED EXCLUSIONS
                    if let Some(name) = e.path().file_name().and_then(|n| n.to_str()) {
                    // Check exact matches
                    if skip_files.contains(&name) {
                        return false;
                    }

                    // Check if the file matches any skip pattern
                    for pattern in skip_files {
                        if pattern.starts_with('.') {
                            // Suffix match
                            if name.ends_with(pattern) {
                                return false;
                            }
                        } else if pattern.ends_with('.') {
                            // Prefix match
                            if name.starts_with(pattern) {
                                return false;
                            }
                        }
                    }

                    // Skip files in known build/generated patterns
                    if name.contains(".gen.")
                        || name.contains("_gen.")
                        || name.contains(".generated.")
                        || name.contains("_generated.")
                        || name.contains(".mock.")
                        || name.contains("_mock.")
                        || name.contains(".g.")
                        || name.contains(".freezed.")
                        || name.contains(".gr.")
                        || name.contains(".reflectable.")
                        || name.contains(".part.")
                    {
                        return false;
                    }

                    // Skip specific Flutter/Dart generated files
                    if name.ends_with(".g.dart")
                        || name.ends_with(".freezed.dart")
                        || name.ends_with(".gr.dart")
                        || name.ends_with(".reflectable.dart")
                        || name.ends_with(".part.dart")
                        || name.ends_with(".gql.dart")
                        || name.ends_with(".graphql.dart")
                        || name.ends_with(".template.dart")
                        || name.ends_with(".tpl.dart")
                    {
                        return false;
                    }
                }

                    // EXTENSION CHECK - ONLY SOURCE CODE EXTENSIONS
                    if let Some(ext) = e.path().extension().and_then(|e| e.to_str()) {
                    // Check if it's in our supported source extensions
                    if source_extensions.contains(&ext) {
                        if let Ok(meta) = e.metadata() {
                            if meta.len() == 0 || meta.len() > config.max_file_size {
                                return false;
                            }
                        }
                        return true;
                    }
                }

                false
            })
            .take(config.max_files)
            .map(|e| e.path().to_path_buf())
            .collect();

        RawProject {
            root: root.to_path_buf(),
            files,
        }
    }
}

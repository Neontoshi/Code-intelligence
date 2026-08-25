// src/engine/file_collector.rs

use crate::engine::config::PipelineConfig;
use crate::engine::stages::RawProject;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

// Use lazy_static for compiled regex patterns
use once_cell::sync::Lazy;
use regex::Regex;

// These regexes are validated at compile time - unwrap is safe
static HASHED_FILE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[a-zA-Z0-9_-]+-[A-Za-z0-9]{6,10}\.(js|css|map)$").unwrap());
static MINIFIED_FILE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\.min\.(js|css)$").unwrap());
static BUNDLED_FILE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"-[A-Za-z0-9]{8,}\.(js|css)$").unwrap());

pub struct FileCollector;

impl FileCollector {
    pub fn collect(root: &Path, config: &PipelineConfig) -> RawProject {
        let skip_dirs = [
            ".git",
            "target",
            "node_modules",
            "__pycache__",
            ".venv",
            "venv",
            ".idea",
            ".vscode",
            ".dart_tool",
            ".pub",
            ".gradle",
            "bin",
            "obj",
            "vendor",
            "remote-dist",
            "assets",
        ];

        let supported_extensions = [
            "rs", "py", "js", "jsx", "ts", "tsx", "go", "java", "dart", "php", "cs", "cpp", "cc",
            "cxx", "hpp", "h",
        ];

        let skip_files = [
            // JavaScript / TypeScript / Node.js
            "package-lock.json",
            "yarn.lock",
            "pnpm-lock.yaml",
            "bun.lockb",
            "bun.lock",
            "npm-shrinkwrap.json",

            // Rust
            "Cargo.lock",

            // Python
            "poetry.lock",
            "Pipfile.lock",
            "pdm.lock",
            "requirements.txt",
            "requirements-dev.txt",

            // Go
            "go.sum",

            // PHP / Composer
            "composer.lock",

            // Dart / Flutter / Pub
            "pubspec.lock",
            ".packages",

            // Java / Gradle / Maven
            "gradle-wrapper.jar",
            "gradle-wrapper.properties",
            "pom.xml.tag",

            // C# / .NET / NuGet
            "packages.lock.json",
            "nuget.config",

            // C++ / CMake / Build metadata
            "CMakeCache.txt",
            "compile_commands.json",

            // Ruby (misc build tools)
            "Gemfile.lock",
        ];

        let files: Vec<PathBuf> = WalkDir::new(root)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_str().unwrap_or("");
                // Skip directories
                if skip_dirs.contains(&name) {
                    return false;
                }

                // Skip files that look like bundled/minified JS
                if let Some(name) = e.file_name().to_str() {
                    if HASHED_FILE_RE.is_match(name)
                        || MINIFIED_FILE_RE.is_match(name)
                        || BUNDLED_FILE_RE.is_match(name)
                    {
                        return false;
                    }
                    // Also skip by heuristic: long name with hash pattern
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

                // Skip files in asset directories (common for bundled code)
                let path_str = e.path().to_string_lossy();
                if path_str.contains("/remote-dist/assets/")
                    || path_str.contains("/dist/assets/")
                    || path_str.contains("/build/assets/")
                {
                    return false;
                }

                if let Some(name) = e.path().file_name().and_then(|n| n.to_str()) {
                    if skip_files.contains(&name) {
                        return false;
                    }
                }

                if let Some(ext) = e.path().extension().and_then(|e| e.to_str()) {
                    if supported_extensions.contains(&ext) {
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

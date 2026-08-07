// src/engine/file_collector.rs

use crate::engine::config::PipelineConfig;
use crate::engine::stages::RawProject;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct FileCollector;

impl FileCollector {
    // src/engine/file_collector.rs

    pub fn collect(root: &Path, config: &PipelineConfig) -> RawProject {
        let skip_dirs = [
            ".git",
            "target",
            "node_modules",
            "__pycache__",
            ".venv",
            "venv",
            "dist",
            "build",
            ".idea",
            ".vscode",
            ".dart_tool",
            ".pub",
            ".gradle",
            "vendor",
            "remote-dist",
            "assets",
        ];

        let supported_extensions = ["rs", "py", "js", "jsx", "ts", "tsx", "go", "java"];

        let skip_files = [
            "package-lock.json",
            "yarn.lock",
            "Cargo.lock",
            "Gemfile.lock",
            "poetry.lock",
            "Pipfile.lock",
            // ⭐ NEW: Skip minified/bundled JS files
            "browser-BXdiCFWD.js",
            "app-ByPOcLMs.js",
            "main-06ciBZDq.js",
            "index-0pYbquBB.js",
            "client-BECxR3b0.js",
            "remote-DmYepkKg.js",
            "butterchunk-CMvS5UXf.js",
        ];

        let files: Vec<PathBuf> = WalkDir::new(root)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_str().unwrap_or("");
                // Skip directories
                if skip_dirs.contains(&name) {
                    return false;
                }

                // ⭐ NEW: Skip files that look like bundled/minified JS
                if name.ends_with(".js")
                    && (name.contains(".min.") || name.contains("-") && name.len() > 30)
                {
                    return false;
                }

                true
            })
            .filter_map(|e| e.ok())
            .filter(|e| {
                if !e.path().is_file() {
                    return false;
                }

                // ⭐ NEW: Skip files in remote-dist/assets/
                let path_str = e.path().to_string_lossy();
                if path_str.contains("/remote-dist/assets/") {
                    return false;
                }
                if path_str.contains("/dist/assets/") {
                    return false;
                }
                if path_str.contains("/build/assets/") {
                    return false;
                }

                if let Some(name) = e.path().file_name().and_then(|n| n.to_str()) {
                    if skip_files.contains(&name) {
                        return false;
                    }
                }

                // ⭐ NEW: Skip minified files by extension pattern
                if let Some(name) = e.path().file_name().and_then(|n| n.to_str()) {
                    if name.ends_with(".min.js") {
                        return false;
                    }
                    // Skip hashed filenames (e.g., "browser-BXdiCFWD.js")
                    if name.ends_with(".js")
                        && name.contains("-")
                        && name.chars().filter(|c| *c == '-').count() >= 1
                    {
                        let parts: Vec<&str> = name.split('-').collect();
                        if parts.len() >= 2 && parts.last().unwrap().len() >= 8 {
                            return false;
                        }
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

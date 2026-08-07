// src/engine/file_collector.rs

use crate::engine::config::PipelineConfig;
use crate::engine::stages::RawProject;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

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
            "dist",
            "build",
            ".idea",
            ".vscode",
            ".dart_tool",
            ".pub",
            ".gradle",
            "vendor",
        ];

        let supported_extensions = ["rs", "py", "js", "jsx", "ts", "tsx", "go", "java"];

        let skip_files = [
            "package-lock.json",
            "yarn.lock",
            "Cargo.lock",
            "Gemfile.lock",
            "poetry.lock",
            "Pipfile.lock",
        ];

        let files: Vec<PathBuf> = WalkDir::new(root)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_str().unwrap_or("");
                !skip_dirs.contains(&name)
            })
            .filter_map(|e| e.ok())
            .filter(|e| {
                if !e.path().is_file() {
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

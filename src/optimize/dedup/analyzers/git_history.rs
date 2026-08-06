use std::collections::HashMap;
use std::path::Path;

pub struct GitHistoryAnalyzer;

impl GitHistoryAnalyzer {
    pub fn analyze_change_patterns(repo_path: &Path) -> HashMap<String, Vec<String>> {
        let mut co_change_graph = HashMap::new();

        if !repo_path.join(".git").exists() {
            return co_change_graph;
        }

        let output = match std::process::Command::new("git")
            .current_dir(repo_path)
            .args(&["log", "--name-only", "--pretty=format:"])
            .output()
        {
            Ok(out) => out,
            Err(_) => return co_change_graph,
        };

        let stdout = match String::from_utf8(output.stdout) {
            Ok(s) => s,
            Err(_) => return co_change_graph,
        };

        let mut current_files: Vec<String> = Vec::new();
        for line in stdout.lines() {
            if line.is_empty() {
                if !current_files.is_empty() {
                    for file in &current_files {
                        co_change_graph
                            .entry(file.clone())
                            .or_insert_with(Vec::new)
                            .extend(current_files.iter().cloned());
                    }
                    current_files.clear();
                }
                continue;
            }

            if line.ends_with(".rs") {
                current_files.push(line.to_string());
            }
        }

        co_change_graph
    }

    pub fn co_change_similarity(
        file_a: &str,
        file_b: &str,
        history: &HashMap<String, Vec<String>>,
    ) -> f64 {
        let default = Vec::new();
        let files_a = history.get(file_a).unwrap_or(&default);
        let files_b = history.get(file_b).unwrap_or(&default);

        let common = files_a.iter().filter(|f| files_b.contains(f)).count();
        let union = files_a.len() + files_b.len() - common;

        if union > 0 {
            common as f64 / union as f64
        } else {
            0.0
        }
    }
}

use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct GitInfo {
    pub file: String,
    pub commits: Vec<CommitInfo>,
    pub total_changes: usize,
    pub authors: Vec<String>,
    pub last_modified: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct CommitInfo {
    pub hash: String,
    pub message: String,
    pub author: String,
    pub date: DateTime<Utc>,
    pub lines_added: usize,
    pub lines_deleted: usize,
}

#[derive(Debug, Clone)]
pub struct GitAnalysis {
    pub files: HashMap<PathBuf, GitInfo>,
    pub top_authors: Vec<(String, usize)>,
    pub total_commits: usize,
    pub most_modified: Vec<(PathBuf, usize)>,
}

pub struct GitAnalyzer;

impl GitAnalyzer {
    /// Analyze git history for a project
    pub fn analyze(root: &Path) -> Result<GitAnalysis, String> {
        if !root.join(".git").exists() {
            return Err("Not a git repository".to_string());
        }

        let mut files = HashMap::new();

        // Get list of tracked files
        let files_output = Self::run_git(
            root,
            &[
                "ls-files", "--", "*.rs", "*.py", "*.js", "*.ts", "*.go", "*.java", "*.dart",
                "*.php", "*.cpp", "*.cs",
            ],
        )?;
        let tracked_files: Vec<&str> = files_output.lines().collect();

        for file in tracked_files {
            if let Ok(info) = Self::analyze_file(root, Path::new(file)) {
                files.insert(PathBuf::from(file), info);
            }
        }

        // Get top authors
        let authors_output = Self::run_git(root, &["shortlog", "-sn"])?;
        let mut authors: Vec<(String, usize)> = authors_output
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split('\t').collect();
                if parts.len() == 2 {
                    let count = parts[0].trim().parse().unwrap_or(0);
                    Some((parts[1].to_string(), count))
                } else {
                    None
                }
            })
            .collect();
        authors.sort_by(|a, b| b.1.cmp(&a.1));

        // Get total commits
        let commit_output = Self::run_git(root, &["rev-list", "--count", "HEAD"])?;
        let total_commits = commit_output.trim().parse().unwrap_or(0);

        // Get most modified files
        let mut file_stats: Vec<(PathBuf, usize)> = files
            .iter()
            .map(|(path, info)| (path.clone(), info.total_changes))
            .collect();
        file_stats.sort_by(|a, b| b.1.cmp(&a.1));

        Ok(GitAnalysis {
            files,
            top_authors: authors,
            total_commits,
            most_modified: file_stats,
        })
    }

    fn analyze_file(root: &Path, file_path: &Path) -> Result<GitInfo, String> {
        let path_str = file_path.to_string_lossy();

        // Get commit history for this file
        let log_output = Self::run_git(
            root,
            &[
                "log",
                "--pretty=format:%H|%s|%an|%ai",
                "--",
                path_str.as_ref(),
            ],
        )?;

        let mut commits = Vec::new();
        for line in log_output.lines() {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() == 4 {
                let date = chrono::DateTime::parse_from_rfc3339(parts[3])
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or(Utc::now());

                // Get changes for this commit
                let diff_output = Self::run_git(
                    root,
                    &[
                        "show",
                        "--numstat",
                        "--format=",
                        parts[0],
                        "--",
                        path_str.as_ref(),
                    ],
                )?;
                let (added, deleted) = Self::parse_diff_stats(&diff_output);

                commits.push(CommitInfo {
                    hash: parts[0].to_string(),
                    message: parts[1].to_string(),
                    author: parts[2].to_string(),
                    date,
                    lines_added: added,
                    lines_deleted: deleted,
                });
            }
        }

        // Get total changes
        let total_changes = commits
            .iter()
            .map(|c| c.lines_added + c.lines_deleted)
            .sum();

        // Get authors
        let mut authors = Vec::new();
        for commit in &commits {
            if !authors.contains(&commit.author) {
                authors.push(commit.author.clone());
            }
        }

        let last_modified = commits.first().map(|c| c.date).unwrap_or(Utc::now());

        Ok(GitInfo {
            file: path_str.to_string(),
            commits,
            total_changes,
            authors,
            last_modified,
        })
    }

    fn parse_diff_stats(output: &str) -> (usize, usize) {
        let parts: Vec<&str> = output.split_whitespace().collect();
        if parts.len() >= 2 {
            let added = parts[0].parse().unwrap_or(0);
            let deleted = parts[1].parse().unwrap_or(0);
            (added, deleted)
        } else {
            (0, 0)
        }
    }

    fn run_git(root: &Path, args: &[&str]) -> Result<String, String> {
        let output = Command::new("git")
            .current_dir(root)
            .args(args)
            .output()
            .map_err(|e| format!("Failed to run git: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Git error: {}", stderr));
        }

        String::from_utf8(output.stdout).map_err(|e| format!("Invalid UTF-8 output: {}", e))
    }
}

impl GitAnalysis {
    pub fn file_activity_score(&self, file: &Path) -> f64 {
        if let Some(info) = self.files.get(file) {
            let recency_bonus = (Utc::now() - info.last_modified).num_days() as f64;
            let max_commits = self
                .files
                .values()
                .map(|f| f.commits.len())
                .max()
                .unwrap_or(1);
            let commit_ratio = info.commits.len() as f64 / max_commits as f64;

            (commit_ratio * 0.7) + (1.0 / (recency_bonus + 1.0) * 0.3)
        } else {
            0.0
        }
    }
}

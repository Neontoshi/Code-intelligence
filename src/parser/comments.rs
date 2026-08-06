use std::path::Path;

#[derive(Debug, Clone)]
pub struct CommentInfo {
    pub line: usize,
    pub content: String,
    pub comment_type: CommentType,
    pub attached_to: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CommentType {
    Line,
    Block,
    Doc,
    Todo,
    FIXME,
    HACK,
    NOTE,
    Warning,
}

pub struct CommentAnalyzer;

impl CommentAnalyzer {
    /// Extract all comments from a file
    pub fn extract_comments(source: &str, path: &Path) -> Vec<CommentInfo> {
        let mut comments = Vec::new();
        let ext = path.extension().unwrap_or_default().to_string_lossy();

        let (single_line, multi_line_start, multi_line_end, doc_prefix) =
            Self::get_comment_patterns(&ext);

        let lines: Vec<&str> = source.lines().collect();
        let mut in_block = false;
        let mut block_content = String::new();
        let mut block_start = 0;

        for (i, line) in lines.iter().enumerate() {
            let line_num = i + 1;
            let trimmed = line.trim();

            // Single line comments
            if !in_block && trimmed.starts_with(&single_line) {
                let content = trimmed.trim_start_matches(&single_line).trim();
                let comment_type = Self::detect_comment_type(content);
                comments.push(CommentInfo {
                    line: line_num,
                    content: content.to_string(),
                    comment_type,
                    attached_to: None,
                });
                continue;
            }

            // Block comments
            if !in_block && trimmed.starts_with(&multi_line_start) {
                in_block = true;
                block_content = trimmed.trim_start_matches(&multi_line_start).to_string();
                block_start = line_num;
                continue;
            }

            if in_block {
                if trimmed.contains(&multi_line_end) {
                    in_block = false;
                    let parts: Vec<&str> = trimmed.split(&multi_line_end).collect();
                    block_content.push_str(parts[0]);

                    let content = block_content.trim();
                    let comment_type = Self::detect_comment_type(content);
                    comments.push(CommentInfo {
                        line: block_start,
                        content: content.to_string(),
                        comment_type,
                        attached_to: None,
                    });
                    block_content.clear();
                } else {
                    block_content.push_str(trimmed);
                    block_content.push(' ');
                }
            }

            // Doc comments
            if trimmed.starts_with(doc_prefix) {
                let content = trimmed.trim_start_matches(doc_prefix).trim();
                comments.push(CommentInfo {
                    line: line_num,
                    content: content.to_string(),
                    comment_type: CommentType::Doc,
                    attached_to: None,
                });
            }
        }

        comments
    }

    fn get_comment_patterns(ext: &str) -> (&'static str, &'static str, &'static str, &'static str) {
        match ext {
            "rs" => ("//", "/*", "*/", "///"),
            "py" => ("#", "\"\"\"", "\"\"\"", "#"),
            "js" | "ts" | "jsx" | "tsx" => ("//", "/*", "*/", "/**"),
            "go" => ("//", "/*", "*/", "//"),
            "java" => ("//", "/*", "*/", "/**"),
            _ => ("//", "/*", "*/", "///"),
        }
    }

    fn detect_comment_type(content: &str) -> CommentType {
        let lower = content.to_lowercase();
        if lower.contains("todo") || lower.contains("to do") {
            CommentType::Todo
        } else if lower.contains("fixme") || lower.contains("fix me") {
            CommentType::FIXME
        } else if lower.contains("hack") || lower.contains("workaround") {
            CommentType::HACK
        } else if lower.contains("note") {
            CommentType::NOTE
        } else if lower.contains("warning") || lower.contains("caution") {
            CommentType::Warning
        } else {
            CommentType::Line
        }
    }

    /// Generate comment statistics
    pub fn comment_stats(comments: &[CommentInfo]) -> CommentStatistics {
        let mut stats = CommentStatistics {
            total_comments: comments.len(),
            doc_comments: 0,
            todo_comments: 0,
            fixme_comments: 0,
            hack_comments: 0,
            note_comments: 0,
            warning_comments: 0,
            line_comments: 0,
        };

        for comment in comments {
            match comment.comment_type {
                CommentType::Doc => stats.doc_comments += 1,
                CommentType::Todo => stats.todo_comments += 1,
                CommentType::FIXME => stats.fixme_comments += 1,
                CommentType::HACK => stats.hack_comments += 1,
                CommentType::NOTE => stats.note_comments += 1,
                CommentType::Warning => stats.warning_comments += 1,
                CommentType::Line | CommentType::Block => stats.line_comments += 1,
            }
        }

        stats
    }

    /// Find TODOs and FIXMEs that need attention
    pub fn find_action_items(comments: &[CommentInfo]) -> Vec<&CommentInfo> {
        comments
            .iter()
            .filter(|c| {
                matches!(
                    c.comment_type,
                    CommentType::Todo | CommentType::FIXME | CommentType::HACK
                )
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct CommentStatistics {
    pub total_comments: usize,
    pub doc_comments: usize,
    pub todo_comments: usize,
    pub fixme_comments: usize,
    pub hack_comments: usize,
    pub note_comments: usize,
    pub warning_comments: usize,
    pub line_comments: usize,
}

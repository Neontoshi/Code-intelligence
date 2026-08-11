use crate::graph::call_graph::{CallGraph, FunctionNode};
use crate::optimize::TokenEstimator;
use crate::parser::tree_sitter::ParsedFile;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Chunk {
    pub id: String,
    pub content: String,
    pub token_count: usize,
    pub metadata: ChunkMetadata,
}

#[derive(Debug, Clone)]
pub struct ChunkMetadata {
    pub file: String,
    pub functions: Vec<String>,
    pub types: Vec<String>,
    pub importance_score: f64,
    pub relationships: Vec<String>,
}

pub struct ChunkStrategy {
    max_tokens: usize,
    overlap_tokens: usize,
    _include_context: bool,
}

impl Default for ChunkStrategy {
    fn default() -> Self {
        Self {
            max_tokens: 4096,
            overlap_tokens: 100,
            _include_context: true,
        }
    }
}

impl ChunkStrategy {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_max_tokens(mut self, max: usize) -> Self {
        self.max_tokens = max;
        self
    }

    pub fn with_overlap(mut self, overlap: usize) -> Self {
        self.overlap_tokens = overlap;
        self
    }

    pub fn chunk_project(&self, call_graph: &CallGraph, files: &[ParsedFile]) -> Vec<Chunk> {
        let mut chunks = Vec::new();

        // Sort functions by importance
        let mut important_functions: Vec<_> = call_graph
            .node_indices()
            .map(|idx| &call_graph[idx])
            .collect();
        important_functions.sort_by(|a, b| b.importance_score.total_cmp(&a.importance_score));

        // Chunk by function significance
        let mut used_functions = HashMap::new();

        // In the chunk_project method, fix the variable scope:
        for func in important_functions {
            if used_functions.contains_key(&func.full_path) {
                continue;
            }

            let mut chunk_content = String::new();
            let mut functions_in_chunk = Vec::new();
            let mut types_in_chunk = Vec::new();

            // Add the function itself
            chunk_content.push_str(&format!(
                "### {} (importance: {:.2})\n",
                func.name, func.importance_score
            ));
            chunk_content.push_str(&self.format_function_with_context(func, call_graph, files));
            functions_in_chunk.push(func.name.clone());

            // Add related functions
            for idx in call_graph.node_indices() {
                let related = &call_graph[idx];
                if used_functions.contains_key(&related.full_path) {
                    continue;
                }

                let current_tokens = TokenEstimator::estimate_tokens(&chunk_content);
                if current_tokens + 200 > self.max_tokens {
                    break;
                }

                // Check if related
                let is_callee = call_graph
                    .get_callees(idx)
                    .iter()
                    .any(|c| c.full_path == func.full_path);
                let is_caller = call_graph
                    .get_callers(idx)
                    .iter()
                    .any(|c| c.full_path == func.full_path);

                if is_callee || is_caller {
                    chunk_content.push_str(&format!("\n### {} (related)\n", related.name));
                    chunk_content
                        .push_str(&self.format_function_with_context(related, call_graph, files));
                    functions_in_chunk.push(related.name.clone());
                    used_functions.insert(related.full_path.clone(), true);
                }
            }

            // Add types from the same file
            if let Some(file) = files.iter().find(|f| f.path == func.file) {
                let current_tokens = TokenEstimator::estimate_tokens(&chunk_content);
                for type_info in &file.types {
                    if current_tokens + 100 > self.max_tokens {
                        break;
                    }
                    if !types_in_chunk.contains(&type_info.name) {
                        chunk_content.push_str(&format!("\n### Type: {}\n", type_info.name));
                        types_in_chunk.push(type_info.name.clone());
                    }
                }
            }

            used_functions.insert(func.full_path.clone(), true);

            // Create the chunk
            let token_count = TokenEstimator::estimate_tokens(&chunk_content);
            let id = format!("chunk_{:04x}", chunks.len());

            chunks.push(Chunk {
                id: id.clone(),
                content: chunk_content,
                token_count,
                metadata: ChunkMetadata {
                    file: func.file.clone(),
                    functions: functions_in_chunk,
                    types: types_in_chunk,
                    importance_score: func.importance_score,
                    relationships: vec![],
                },
            });
        }

        chunks
    }

    fn format_function_with_context(
        &self,
        func: &FunctionNode,
        call_graph: &CallGraph,
        _files: &[ParsedFile],
    ) -> String {
        let mut output = String::new();

        output.push_str(&format!("- File: {}\n", func.file));
        output.push_str(&format!("- Line: {}\n", func.line));
        output.push_str(&format!("- Public: {}\n", func.is_public));
        output.push_str(&format!("- Async: {}\n", func.is_async));
        output.push_str(&format!("- Complexity: {:.2}\n", func.complexity));

        if !func.params.is_empty() {
            output.push_str("- Parameters:\n");
            for param in &func.params {
                output.push_str(&format!("  - {}\n", param));
            }
        }

        if !func.returns.is_empty() {
            output.push_str(&format!("- Returns: {}\n", func.returns.join(", ")));
        }

        if let Some(doc) = &func.doc_comment {
            output.push_str(&format!("- Documentation: {}\n", doc));
        }

        // Add call relationships
        let idx = call_graph
            .node_indices()
            .find(|i| call_graph[*i].full_path == func.full_path);

        if let Some(idx) = idx {
            let callees = call_graph.get_callees(idx);
            let callers = call_graph.get_callers(idx);

            if !callees.is_empty() {
                output.push_str("- Calls:\n");
                for callee in callees {
                    output.push_str(&format!("  - {}\n", callee.name));
                }
            }

            if !callers.is_empty() {
                output.push_str("- Called by:\n");
                for caller in callers {
                    output.push_str(&format!("  - {}\n", caller.name));
                }
            }
        }

        output
    }

    /// Chunk by file boundaries
    pub fn chunk_by_file(&self, files: &[ParsedFile]) -> Vec<Chunk> {
        let mut chunks = Vec::new();

        for file in files {
            if file.functions.is_empty() && file.types.is_empty() {
                continue;
            }

            let mut content = String::new();
            content.push_str(&format!("# File: {}\n\n", file.path));
            content.push_str(&format!("Language: {}\n\n", file.language));

            // Add types
            for type_info in &file.types {
                content.push_str(&format!("## Type: {}\n", type_info.name));
                content.push_str(&format!("Kind: {:?}\n", type_info.kind));
                content.push_str(&format!("Line: {}\n\n", type_info.line));
            }

            // Add functions
            for func in &file.functions {
                content.push_str(&format!("## Function: {}\n", func.name));
                content.push_str(&format!("Line: {}\n", func.line));
                if let Some(doc) = &func.doc_comment {
                    content.push_str(&format!("Doc: {}\n", doc));
                }
                content.push('\n');
            }

            let token_count = TokenEstimator::estimate_tokens(&content);
            let functions: Vec<_> = file.functions.iter().map(|f| f.name.clone()).collect();
            let types: Vec<_> = file.types.iter().map(|t| t.name.clone()).collect();

            chunks.push(Chunk {
                id: format!("file_{:04x}", chunks.len()),
                content,
                token_count,
                metadata: ChunkMetadata {
                    file: file.path.clone(),
                    functions,
                    types,
                    importance_score: 0.5,
                    relationships: vec![],
                },
            });
        }

        chunks
    }
}

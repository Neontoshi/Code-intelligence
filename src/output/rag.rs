use crate::graph::call_graph::{CallGraph, FunctionNode};
use crate::parser::tree_sitter::ParsedFile;

#[derive(Debug, Clone)]
pub struct RAGChunk {
    pub id: String,
    pub content: String,
    pub metadata: RAGMetadata,
}

#[derive(Debug, Clone)]
pub struct RAGMetadata {
    pub file: String,
    pub function: String,
    pub line: usize,
    pub importance: f64,
    pub tags: Vec<String>,
    pub embedding_context: String,
}

pub struct RAGGenerator;

impl RAGGenerator {
    /// Generate RAG chunks from codebase
    pub fn generate_chunks(call_graph: &CallGraph, files: &[ParsedFile]) -> Vec<RAGChunk> {
        let mut chunks = Vec::new();

        for idx in call_graph.node_indices() {
            let func = &call_graph[idx];
            let chunk = Self::create_chunk(func, call_graph, files);
            chunks.push(chunk);
        }

        chunks
    }

    fn create_chunk(
        func: &FunctionNode,
        call_graph: &CallGraph,
        _files: &[ParsedFile],
    ) -> RAGChunk {
        let mut content = String::new();
        let mut tags = Vec::new();

        // Function header
        content.push_str(&format!("# Function: {}\n\n", func.name));

        // Metadata
        content.push_str(&format!("**File**: `{}`\n", func.file));
        content.push_str(&format!("**Line**: {}\n", func.line));
        content.push_str(&format!("**Importance**: {:.2}\n", func.importance_score));
        content.push_str(&format!("**Public**: {}\n", func.is_public));
        content.push_str(&format!("**Async**: {}\n", func.is_async));
        content.push_str(&format!("**Complexity**: {:.2}\n", func.complexity));
        content.push('\n');

        // Documentation
        if let Some(doc) = &func.doc_comment {
            content.push_str("## Documentation\n\n");
            content.push_str(doc);
            content.push_str("\n\n");
        }

        // Signature
        content.push_str("## Signature\n\n");
        content.push_str(&format!("```rust\nfn {}(", func.name));
        let params: Vec<_> = func.params.iter().map(|p| format!("{}", p)).collect();
        content.push_str(&params.join(", "));
        content.push_str(")");
        if !func.returns.is_empty() {
            content.push_str(&format!(" -> {}", func.returns.join(", ")));
        }
        content.push_str("\n```\n\n");

        // Relationships
        let idx = call_graph
            .node_indices()
            .find(|i| call_graph[*i].full_path == func.full_path);

        if let Some(idx) = idx {
            let callees = call_graph.get_callees(idx);
            let callers = call_graph.get_callers(idx);

            if !callees.is_empty() {
                content.push_str("## Calls\n\n");
                for callee in callees {
                    content.push_str(&format!("- `{}`\n", callee.name));
                    tags.push(callee.name.clone());
                }
                content.push('\n');
            }

            if !callers.is_empty() {
                content.push_str("## Called By\n\n");
                for caller in callers {
                    content.push_str(&format!("- `{}`\n", caller.name));
                }
                content.push('\n');
            }
        }

        // Tags
        tags.push(func.name.clone());
        tags.push(func.file.clone());
        if func.is_public {
            tags.push("public".to_string());
        }
        if func.is_async {
            tags.push("async".to_string());
        }

        // Create embedding context
        let embedding_context = Self::create_embedding_context(func);

        RAGChunk {
            id: format!(
                "rag_{}",
                func.full_path.replace("::", "_").replace('/', "_")
            ),
            content,
            metadata: RAGMetadata {
                file: func.file.clone(),
                function: func.name.clone(),
                line: func.line,
                importance: func.importance_score,
                tags,
                embedding_context,
            },
        }
    }

    fn create_embedding_context(func: &FunctionNode) -> String {
        let mut context = String::new();

        context.push_str(&format!("Function {} ", func.name));
        context.push_str(&format!("in file {} ", func.file));

        if let Some(doc) = &func.doc_comment {
            context.push_str(&format!("with documentation: {} ", doc));
        }

        if func.is_public {
            context.push_str("public ");
        }
        if func.is_async {
            context.push_str("async ");
        }

        context.push_str(&format!("with {} parameters ", func.params.len()));

        if !func.returns.is_empty() {
            context.push_str(&format!("returning {}", func.returns.join(", ")));
        }

        context
    }

    /// Generate RAG-optimized markdown
    pub fn generate_rag_markdown(call_graph: &CallGraph, files: &[ParsedFile]) -> String {
        let mut output = String::new();
        let chunks = Self::generate_chunks(call_graph, files);

        output.push_str("# RAG Documentation\n\n");
        output.push_str("This documentation is optimized for retrieval-augmented generation.\n\n");
        output.push_str("---\n\n");

        // Table of contents
        output.push_str("## Table of Contents\n\n");
        for chunk in &chunks {
            output.push_str(&format!("- [{}](#{})\n", chunk.metadata.function, chunk.id));
        }
        output.push('\n');
        output.push_str("---\n\n");

        // Chunks
        for chunk in chunks {
            output.push_str(&format!("## {}\n\n", chunk.metadata.function));
            output.push_str(&chunk.content);
            output.push_str("---\n\n");
        }

        output
    }

    /// Generate a JSONL file for RAG training
    pub fn generate_training_data(call_graph: &CallGraph, files: &[ParsedFile]) -> String {
        let chunks = Self::generate_chunks(call_graph, files);
        let mut output = String::new();

        for chunk in chunks {
            let entry = serde_json::json!({
                "id": chunk.id,
                "content": chunk.content,
                "metadata": {
                    "file": chunk.metadata.file,
                    "function": chunk.metadata.function,
                    "line": chunk.metadata.line,
                    "importance": chunk.metadata.importance,
                    "tags": chunk.metadata.tags,
                    "embedding_context": chunk.metadata.embedding_context,
                }
            });

            output.push_str(&serde_json::to_string(&entry).unwrap());
            output.push('\n');
        }

        output
    }
}

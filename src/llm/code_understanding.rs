// src/llm/code_understanding.rs

use crate::graph::call_graph::{CallGraph, FunctionNode};
use crate::graph::traits::GraphMetrics;
use crate::llm::extract_json_from_response;
use crate::llm::{GenerationOptions, LLMMessage, LLMProvider};
use crate::parser::tree_sitter::ParsedFile;

use std::collections::HashMap;
use std::sync::Arc;

// ============================================================================
// Code Understanding Engine
// ============================================================================

/// Main engine for LLM-powered code understanding
#[derive(Clone)]
pub struct CodeUnderstandingEngine {
    provider: Arc<dyn LLMProvider>,
    cache: HashMap<String, String>, // Simple in-memory cache
}

impl CodeUnderstandingEngine {
    pub fn new(provider: Arc<dyn LLMProvider>) -> Self {
        Self {
            provider,
            cache: HashMap::new(),
        }
    }

    /// Clear the cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get cache size
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }

    // ========================================================================
    // Function Analysis
    // ========================================================================

    /// Generate a concise summary of a function
    pub async fn summarize_function(
        &mut self,
        func: &FunctionNode,
        source: &str,
    ) -> Result<String, String> {
        // Check cache
        let cache_key = format!("summary_{}", func.full_path);
        if let Some(cached) = self.cache.get(&cache_key) {
            return Ok(cached.clone());
        }

        // Truncate source if too long
        let source_preview = Self::truncate_source(source, 500);

        let prompt = format!(
            r#"Summarize this function concisely (1-2 sentences). Focus on WHAT it does, not HOW.

Function: {}
File: {}
Parameters: {:?}
Returns: {:?}
Public: {}
Async: {}
Complexity: {:.2}

Code:
```
{}
```

Summary (1-2 sentences):"#,
            func.name,
            func.file,
            func.params.iter().map(|p| p.as_str()).collect::<Vec<_>>(),
            func.returns,
            func.is_public,
            func.is_async,
            func.complexity,
            source_preview
        );

        let messages = vec![
            LLMMessage::system(
                "You are a code documentation expert. Provide concise, clear function summaries.",
            ),
            LLMMessage::user(prompt),
        ];

        let response = self.generate_with_options(&messages, 0.3, 200).await?;

        // Cache the result
        self.cache.insert(cache_key, response.clone());

        Ok(response)
    }

    /// Explain a function in detail
    pub async fn explain_function(
        &mut self,
        func: &FunctionNode,
        source: &str,
        question: Option<&str>,
    ) -> Result<String, String> {
        // Check cache
        let cache_key = format!(
            "explain_{}_{}",
            func.full_path,
            question.unwrap_or("general")
        );
        if let Some(cached) = self.cache.get(&cache_key) {
            return Ok(cached.clone());
        }

        let source_preview = Self::truncate_source(source, 800);

        let question_part = if let Some(q) = question {
            format!("\nQuestion: {}", q)
        } else {
            "\nExplain what this function does, how it works, and any important details."
                .to_string()
        };

        let prompt = format!(
            r#"Explain this function in detail.

Function: {}
File: {}
Parameters: {:?}
Returns: {:?}
Public: {}
Async: {}
Complexity: {:.2}

Code:
```
{}
```
{}

Explanation:"#,
            func.name,
            func.file,
            func.params.iter().map(|p| p.as_str()).collect::<Vec<_>>(),
            func.returns,
            func.is_public,
            func.is_async,
            func.complexity,
            source_preview,
            question_part
        );

        let messages = vec![
            LLMMessage::system(
                "You are a code analysis expert. Provide clear, thorough explanations.",
            ),
            LLMMessage::user(prompt),
        ];

        let response = self.generate_with_options(&messages, 0.3, 1000).await?;

        // Cache the result
        self.cache.insert(cache_key, response.clone());

        Ok(response)
    }

    /// Find potential bugs or issues in a function
    pub async fn analyze_bugs(
        &mut self,
        func: &FunctionNode,
        source: &str,
    ) -> Result<Vec<CodeIssue>, String> {
        let source_preview = Self::truncate_source(source, 600);

        let prompt = format!(
            r#"Analyze this function for potential bugs, edge cases, and issues.

Function: {}
File: {}
Parameters: {:?}
Returns: {:?}

Code:
```
{}
```

Provide analysis in JSON format:
{{
    "issues": [
        {{
            "severity": "high|medium|low",
            "category": "logic|performance|security|error_handling|style",
            "description": "What the issue is",
            "suggestion": "How to fix it",
            "line": "Line number if known or 'unknown'"
        }}
    ]
}}"#,
            func.name,
            func.file,
            func.params.iter().map(|p| p.as_str()).collect::<Vec<_>>(),
            func.returns,
            source_preview
        );

        let messages = vec![
            LLMMessage::system("You are a code quality expert. Identify issues and provide fix suggestions. Respond with valid JSON only."),
            LLMMessage::user(prompt),
        ];

        let response = self.generate_structured(&messages, 0.1, 1000).await?;

        // Parse the response
        if let Some(issues_array) = response["issues"].as_array() {
            let mut code_issues = Vec::new();
            for item in issues_array {
                if let (Some(severity), Some(category), Some(description), Some(suggestion)) = (
                    item["severity"].as_str(),
                    item["category"].as_str(),
                    item["description"].as_str(),
                    item["suggestion"].as_str(),
                ) {
                    code_issues.push(CodeIssue {
                        severity: severity.to_string(),
                        category: category.to_string(),
                        description: description.to_string(),
                        suggestion: suggestion.to_string(),
                        line: item["line"].as_str().unwrap_or("unknown").to_string(),
                    });
                }
            }
            return Ok(code_issues);
        }

        Ok(Vec::new())
    }

    /// Suggest improvements for a function
    pub async fn suggest_improvements(
        &mut self,
        func: &FunctionNode,
        source: &str,
    ) -> Result<Vec<CodeSuggestion>, String> {
        let source_preview = Self::truncate_source(source, 600);

        let prompt = format!(
            r#"Suggest improvements for this function.

Function: {}
File: {}
Parameters: {:?}
Returns: {:?}

Code:
```
{}
```

Provide suggestions in JSON format:
{{
    "suggestions": [
        {{
            "type": "performance|readability|maintainability|architecture|error_handling",
            "description": "What to improve",
            "reason": "Why this improvement helps",
            "example": "Optional code example"
        }}
    ]
}}"#,
            func.name,
            func.file,
            func.params.iter().map(|p| p.as_str()).collect::<Vec<_>>(),
            func.returns,
            source_preview
        );

        let messages = vec![
            LLMMessage::system("You are a code optimization expert. Suggest improvements with clear reasoning. Respond with valid JSON only."),
            LLMMessage::user(prompt),
        ];

        let response = self.generate_structured(&messages, 0.2, 1000).await?;

        if let Some(suggestions_array) = response["suggestions"].as_array() {
            let mut suggestions = Vec::new();
            for item in suggestions_array {
                if let (Some(imp_type), Some(description), Some(reason)) = (
                    item["type"].as_str(),
                    item["description"].as_str(),
                    item["reason"].as_str(),
                ) {
                    suggestions.push(CodeSuggestion {
                        suggestion_type: imp_type.to_string(),
                        description: description.to_string(),
                        reason: reason.to_string(),
                        example: item["example"].as_str().map(|s| s.to_string()),
                    });
                }
            }
            return Ok(suggestions);
        }

        Ok(Vec::new())
    }

    // ========================================================================
    // Project-Level Analysis
    // ========================================================================
    pub async fn generate_documentation(
        &mut self,
        call_graph: &CallGraph,
        files: &[ParsedFile],
    ) -> Result<String, String> {
        // Build languages inline
        let languages: Vec<String> = files.iter().map(|f| f.language.clone()).collect();
        let mut langs = languages;
        langs.sort();
        langs.dedup();
        let langs_str = langs.join(", ");

        // Build key files inline
        let mut key_files_output = String::new();
        for file in files.iter().take(5) {
            let func_count = file.functions.len();
            let type_count = file.types.len();
            key_files_output.push_str(&format!(
                "  - {} ({} functions, {} types)\n",
                file.path.split('/').last().unwrap_or(&file.path),
                func_count,
                type_count
            ));
        }

        // Build project context
        let prompt = format!(
            r#"Generate comprehensive documentation for this codebase.

    ## Project Context

    Total Functions: {}
    Total Files: {}
    Languages: {}

    ## Architecture Overview

    Most Important Functions (by importance):
    {}

    ## Key Files
    {}

    ## Entry Points
    {}

    ## Complexity Summary
    {}

    Based on this context, generate:
    1. A high-level overview of what this codebase does (2-3 paragraphs)
    2. The main components and their responsibilities
    3. How data flows through the system
    4. Key architectural patterns used
    5. Important entry points and APIs
    6. Dependencies and external integrations
    7. Potential areas for improvement

    ## Documentation
    "#,
            call_graph.node_count(),
            files.len(),
            langs_str,
            self.get_important_functions(call_graph, 10),
            key_files_output,
            self.get_entry_points(call_graph, 5),
            self.get_complexity_summary(call_graph)
        );

        let messages = vec![
            LLMMessage::system("You are a software architect and technical writer. Generate clear, professional documentation with markdown formatting."),
            LLMMessage::user(prompt),
        ];

        self.generate_with_options(&messages, 0.5, 1000).await
    }
    /// Analyze project architecture
    pub async fn analyze_architecture(
        &mut self,
        call_graph: &CallGraph,
        files: &[ParsedFile],
    ) -> Result<ArchitectureAnalysis, String> {
        let context = self.build_project_context(call_graph, files);

        let prompt = format!(
            r#"Analyze the architecture of this codebase.

## Context
{}

Provide analysis in JSON format:
{{
    "layers": [
        {{
            "name": "Layer name",
            "responsibility": "What this layer does",
            "components": ["Component1", "Component2"]
        }}
    ],
    "patterns": [
        {{
            "name": "Pattern name",
            "description": "How it's used",
            "locations": ["Where it's applied"]
        }}
    ],
    "data_flow": "Description of how data flows through the system",
    "coupling": {{
        "score": 0.0-1.0,
        "description": "Coupling analysis"
    }},
    "cohesion": {{
        "score": 0.0-1.0,
        "description": "Cohesion analysis"
    }},
    "recommendations": ["Recommendation 1", "Recommendation 2"]
}}"#,
            context
        );

        let messages = vec![
            LLMMessage::system("You are a software architecture expert. Analyze architecture and provide structured output. Respond with valid JSON only."),
            LLMMessage::user(prompt),
        ];

        let response = self.generate_structured(&messages, 0.2, 2000).await?;

        // Parse the response into a structured type
        let mut analysis = ArchitectureAnalysis {
            layers: Vec::new(),
            patterns: Vec::new(),
            data_flow: String::new(),
            coupling: AnalysisScore::default(),
            cohesion: AnalysisScore::default(),
            recommendations: Vec::new(),
        };

        if let Some(layers) = response["layers"].as_array() {
            for layer in layers {
                if let (Some(name), Some(responsibility)) =
                    (layer["name"].as_str(), layer["responsibility"].as_str())
                {
                    let components: Vec<String> = layer["components"]
                        .as_array()
                        .unwrap_or(&vec![])
                        .iter()
                        .filter_map(|c| c.as_str().map(|s| s.to_string()))
                        .collect();

                    analysis.layers.push(ArchitectureLayer {
                        name: name.to_string(),
                        responsibility: responsibility.to_string(),
                        components,
                    });
                }
            }
        }

        if let Some(patterns) = response["patterns"].as_array() {
            for pattern in patterns {
                if let (Some(name), Some(description)) =
                    (pattern["name"].as_str(), pattern["description"].as_str())
                {
                    let locations: Vec<String> = pattern["locations"]
                        .as_array()
                        .unwrap_or(&vec![])
                        .iter()
                        .filter_map(|l| l.as_str().map(|s| s.to_string()))
                        .collect();

                    analysis.patterns.push(ArchitecturePattern {
                        name: name.to_string(),
                        description: description.to_string(),
                        locations,
                    });
                }
            }
        }

        if let Some(flow) = response["data_flow"].as_str() {
            analysis.data_flow = flow.to_string();
        }

        if let Some(coupling) = response["coupling"].as_object() {
            analysis.coupling = AnalysisScore {
                score: coupling["score"].as_f64().unwrap_or(0.5),
                description: coupling["description"].as_str().unwrap_or("").to_string(),
            };
        }

        if let Some(cohesion) = response["cohesion"].as_object() {
            analysis.cohesion = AnalysisScore {
                score: cohesion["score"].as_f64().unwrap_or(0.5),
                description: cohesion["description"].as_str().unwrap_or("").to_string(),
            };
        }

        if let Some(recommendations) = response["recommendations"].as_array() {
            for rec in recommendations {
                if let Some(text) = rec.as_str() {
                    analysis.recommendations.push(text.to_string());
                }
            }
        }

        Ok(analysis)
    }

    /// Generate a README for the project
    pub async fn generate_readme(
        &mut self,
        call_graph: &CallGraph,
        files: &[ParsedFile],
        project_name: &str,
    ) -> Result<String, String> {
        let context = self.build_project_context(call_graph, files);

        let prompt = format!(
            r#"Generate a professional README.md for this project.

Project Name: {}
Context: {}

Generate a README with:
1. Project title and description
2. Features
3. Installation instructions
4. Usage examples
5. API documentation overview
6. Contributing guidelines
7. License information

Use markdown formatting with proper headings and code blocks."#,
            project_name, context
        );

        let messages = vec![
            LLMMessage::system(
                "You are a technical writer. Generate professional, well-structured README files.",
            ),
            LLMMessage::user(prompt),
        ];

        self.generate_with_options(&messages, 0.4, 2000).await
    }

    // ========================================================================
    // Duplicate Analysis with LLM
    // ========================================================================

    /// Analyze if two functions are duplicates using LLM
    pub async fn analyze_duplicate_pair(
        &mut self,
        func_a: &FunctionNode,
        func_b: &FunctionNode,
        source_a: &str,
        source_b: &str,
    ) -> Result<DuplicateAnalysisResult, String> {
        let source_a_preview = Self::truncate_source(source_a, 400);
        let source_b_preview = Self::truncate_source(source_b, 400);

        let prompt = format!(
            r#"Analyze if these two functions are duplicates (similar enough to be refactored).

## Function A
Name: {}
File: {}
Parameters: {:?}
Returns: {:?}
Public: {}
Async: {}
Complexity: {:.2}

```rust
{}
```

## Function B
Name: {}
File: {}
Parameters: {:?}
Returns: {:?}
Public: {}
Async: {}
Complexity: {:.2}

```rust
{}
```

Provide analysis in JSON format:
{{
    "is_duplicate": true/false,
    "confidence": 0.0-1.0,
    "similarity_score": 0.0-1.0,
    "reasoning": "Why they are or aren't duplicates",
    "key_differences": ["Difference 1", "Difference 2"],
    "refactoring_suggestion": "How to refactor if duplicates",
    "refactoring_effort": "low|medium|high"
}}"#,
            func_a.name,
            func_a.file,
            func_a.params.iter().map(|p| p.as_str()).collect::<Vec<_>>(),
            func_a.returns,
            func_a.is_public,
            func_a.is_async,
            func_a.complexity,
            source_a_preview,
            func_b.name,
            func_b.file,
            func_b.params.iter().map(|p| p.as_str()).collect::<Vec<_>>(),
            func_b.returns,
            func_b.is_public,
            func_b.is_async,
            func_b.complexity,
            source_b_preview
        );

        let messages = vec![
            LLMMessage::system("You are a code analysis expert. Analyze duplicate code with high accuracy. Respond with valid JSON only."),
            LLMMessage::user(prompt),
        ];

        let response = self.generate_structured(&messages, 0.1, 800).await?;

        Ok(DuplicateAnalysisResult {
            is_duplicate: response["is_duplicate"].as_bool().unwrap_or(false),
            confidence: response["confidence"].as_f64().unwrap_or(0.0),
            similarity_score: response["similarity_score"].as_f64().unwrap_or(0.0),
            reasoning: response["reasoning"].as_str().unwrap_or("").to_string(),
            key_differences: response["key_differences"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|d| d.as_str().map(|s| s.to_string()))
                .collect(),
            refactoring_suggestion: response["refactoring_suggestion"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            refactoring_effort: response["refactoring_effort"]
                .as_str()
                .unwrap_or("medium")
                .to_string(),
        })
    }

    // ========================================================================
    // Helper Methods
    // ========================================================================

    /// Generate with custom temperature and max tokens
    async fn generate_with_options(
        &self,
        messages: &[LLMMessage],
        temperature: f32,
        max_tokens: usize,
    ) -> Result<String, String> {
        let options = GenerationOptions {
            temperature,
            max_tokens,
            ..Default::default()
        };

        match self.provider.generate(messages, &options).await {
            Ok(response) => Ok(response.content.trim().to_string()),
            Err(e) => Err(e.to_string()),
        }
    }

    /// Generate structured JSON response
    async fn generate_structured(
        &self,
        messages: &[LLMMessage],
        temperature: f32,
        max_tokens: usize,
    ) -> Result<serde_json::Value, String> {
        let options = GenerationOptions {
            temperature,
            max_tokens,
            ..Default::default()
        };

        let response = match self.provider.generate(messages, &options).await {
            Ok(r) => r,
            Err(e) => return Err(e.to_string()),
        };

        extract_json_from_response(&response.content)
            .map_err(|e| format!("Failed to parse JSON: {}", e))
    }

    /// Truncate source code to a reasonable length
    fn truncate_source(source: &str, max_chars: usize) -> String {
        if source.len() <= max_chars {
            return source.to_string();
        }

        // Try to truncate at a reasonable boundary
        let truncated = &source[..max_chars];
        if let Some(last_newline) = truncated.rfind('\n') {
            format!("{}\n... (truncated)", &truncated[..last_newline])
        } else {
            format!("{}... (truncated)", truncated)
        }
    }

    /// Build project context for LLM prompts
    fn build_project_context(&self, call_graph: &CallGraph, files: &[ParsedFile]) -> String {
        let mut context = String::new();

        context.push_str(&format!("Total Functions: {}\n", call_graph.node_count()));
        context.push_str(&format!("Total Files: {}\n", files.len()));
        context.push_str(&format!("Languages: {}\n\n", {
            let languages: Vec<String> = files.iter().map(|f| f.language.clone()).collect();
            let mut langs = languages;
            langs.sort();
            langs.dedup();
            langs.join(", ")
        }));

        // Important functions
        context.push_str("Most Important Functions:\n");
        let important = self.get_important_functions(call_graph, 5);
        context.push_str(&important);
        context.push('\n');

        // Entry points
        context.push_str("Entry Points:\n");
        let entries = self.get_entry_points(call_graph, 5);
        context.push_str(&entries);
        context.push('\n');

        // Complexity summary
        context.push_str(&self.get_complexity_summary(call_graph));

        context
    }

    /// Get most important functions as a string
    fn get_important_functions(&self, call_graph: &CallGraph, count: usize) -> String {
        let mut sorted: Vec<_> = call_graph
            .node_indices()
            .map(|idx| &call_graph[idx])
            .collect();
        sorted.sort_by(|a, b| {
            b.importance_score
                .partial_cmp(&a.importance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut output = String::new();
        for func in sorted.iter().take(count) {
            output.push_str(&format!(
                "  - {} (score: {:.2}, file: {})\n",
                func.name,
                func.importance_score,
                func.file.split('/').last().unwrap_or(&func.file)
            ));
        }
        output
    }

    /// Get entry points as a string
    fn get_entry_points(&self, call_graph: &CallGraph, count: usize) -> String {
        let mut entries: Vec<_> = call_graph
            .node_indices()
            .filter(|idx| {
                let func = &call_graph[*idx];
                func.is_public && call_graph.get_callers(*idx).is_empty()
            })
            .map(|idx| &call_graph[idx])
            .collect();
        entries.sort_by(|a, b| {
            b.importance_score
                .partial_cmp(&a.importance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut output = String::new();
        for func in entries.iter().take(count) {
            output.push_str(&format!(
                "  - {} (file: {}, importance: {:.2})\n",
                func.name,
                func.file.split('/').last().unwrap_or(&func.file),
                func.importance_score
            ));
        }
        output
    }

    /// Get complexity summary
    fn get_complexity_summary(&self, call_graph: &CallGraph) -> String {
        let mut complexities: Vec<f64> = call_graph
            .node_indices()
            .map(|idx| call_graph[idx].complexity)
            .collect();

        if complexities.is_empty() {
            return "No functions found.\n".to_string();
        }

        complexities.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let sum: f64 = complexities.iter().sum();
        let avg = sum / complexities.len() as f64;
        let max = complexities.last().unwrap_or(&0.0);
        let min = complexities.first().unwrap_or(&0.0);

        format!(
            "Complexity Summary:\n  - Average: {:.2}\n  - Max: {:.2}\n  - Min: {:.2}\n",
            avg, max, min
        )
    }
}

// ============================================================================
// Supporting Types
// ============================================================================

/// A code issue detected by the LLM
#[derive(Debug, Clone)]
pub struct CodeIssue {
    pub severity: String, // high, medium, low
    pub category: String, // logic, performance, security, error_handling, style
    pub description: String,
    pub suggestion: String,
    pub line: String,
}

/// A code suggestion from the LLM
#[derive(Debug, Clone)]
pub struct CodeSuggestion {
    pub suggestion_type: String, // performance, readability, maintainability, architecture, error_handling
    pub description: String,
    pub reason: String,
    pub example: Option<String>,
}

/// Result of duplicate analysis
#[derive(Debug, Clone)]
pub struct DuplicateAnalysisResult {
    pub is_duplicate: bool,
    pub confidence: f64,
    pub similarity_score: f64,
    pub reasoning: String,
    pub key_differences: Vec<String>,
    pub refactoring_suggestion: String,
    pub refactoring_effort: String, // low, medium, high
}

/// Architecture analysis results
#[derive(Debug, Clone)]
pub struct ArchitectureAnalysis {
    pub layers: Vec<ArchitectureLayer>,
    pub patterns: Vec<ArchitecturePattern>,
    pub data_flow: String,
    pub coupling: AnalysisScore,
    pub cohesion: AnalysisScore,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ArchitectureLayer {
    pub name: String,
    pub responsibility: String,
    pub components: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ArchitecturePattern {
    pub name: String,
    pub description: String,
    pub locations: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct AnalysisScore {
    pub score: f64,
    pub description: String,
}

impl Default for AnalysisScore {
    fn default() -> Self {
        Self {
            score: 0.5,
            description: String::new(),
        }
    }
}

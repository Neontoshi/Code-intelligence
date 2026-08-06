// src/llm/prompts.rs

//! Centralized Prompt Templates for LLM Code Analysis
//!
//! This module provides reusable prompt templates for various code analysis tasks.
//! All prompts are designed to work with phi-2 and other small models.

use crate::graph::call_graph::FunctionNode;

// ============================================================================
// System Prompts
// ============================================================================

/// System prompts for different roles
pub mod system {
    /// General code assistant
    pub const CODE_ASSISTANT: &str =
        "You are a helpful code analysis assistant. Provide clear, accurate responses.";

    /// Code documentation expert
    pub const DOCUMENTATION_EXPERT: &str =
        "You are a code documentation expert. Write clear, concise documentation.";

    /// Code quality expert
    pub const QUALITY_EXPERT: &str =
        "You are a code quality expert. Identify issues and provide actionable suggestions.";

    /// Architecture expert
    pub const ARCHITECTURE_EXPERT: &str =
        "You are a software architecture expert. Analyze code structure and provide insights.";

    /// Code optimization expert
    pub const OPTIMIZATION_EXPERT: &str =
        "You are a code optimization expert. Suggest performance and readability improvements.";

    /// Duplicate detection expert
    pub const DUPLICATE_EXPERT: &str =
        "You are a duplicate code detection expert. Accurately identify code duplication.";

    /// Technical writer
    pub const TECHNICAL_WRITER: &str =
        "You are a technical writer. Generate professional, well-structured documentation.";
}

// ============================================================================
// Prompt Builders
// ============================================================================

/// Build prompts for various code analysis tasks
pub struct PromptBuilder;

impl PromptBuilder {
    // ========================================================================
    // Function Analysis Prompts
    // ========================================================================

    /// Build a prompt for function summarization
    pub fn summarize_function(func: &FunctionNode, source_preview: &str) -> String {
        format!(
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
        )
    }

    /// Build a prompt for detailed function explanation
    pub fn explain_function(
        func: &FunctionNode,
        source_preview: &str,
        question: Option<&str>,
    ) -> String {
        let question_part = if let Some(q) = question {
            format!("\nQuestion: {}", q)
        } else {
            "\nExplain what this function does, how it works, and any important details."
                .to_string()
        };

        format!(
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
        )
    }

    /// Build a prompt for bug analysis
    pub fn analyze_bugs(func: &FunctionNode, source_preview: &str) -> String {
        format!(
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
        )
    }

    /// Build a prompt for improvement suggestions
    pub fn suggest_improvements(func: &FunctionNode, source_preview: &str) -> String {
        format!(
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
        )
    }

    /// Build a prompt for duplicate analysis
    pub fn analyze_duplicate(
        func_a: &FunctionNode,
        func_b: &FunctionNode,
        source_a: &str,
        source_b: &str,
    ) -> String {
        format!(
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
            source_a,
            func_b.name,
            func_b.file,
            func_b.params.iter().map(|p| p.as_str()).collect::<Vec<_>>(),
            func_b.returns,
            func_b.is_public,
            func_b.is_async,
            func_b.complexity,
            source_b
        )
    }

    // ========================================================================
    // Project-Level Prompts
    // ========================================================================

    /// Build a prompt for project documentation
    pub fn generate_documentation(
        total_functions: usize,
        total_files: usize,
        languages: &[String],
        important_functions: &str,
        key_files: &str,
        entry_points: &str,
        complexity_summary: &str,
    ) -> String {
        format!(
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
            total_functions,
            total_files,
            languages.join(", "),
            important_functions,
            key_files,
            entry_points,
            complexity_summary
        )
    }

    /// Build a prompt for architecture analysis
    pub fn analyze_architecture(context: &str) -> String {
        format!(
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
        )
    }

    /// Build a prompt for README generation
    pub fn generate_readme(project_name: &str, context: &str) -> String {
        format!(
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
        )
    }

    // ========================================================================
    // Code Review Prompts
    // ========================================================================

    /// Build a prompt for code review
    pub fn code_review(func: &FunctionNode, source_preview: &str) -> String {
        format!(
            r#"Review this function and provide feedback.

Function: {}
File: {}
Parameters: {:?}
Returns: {:?}
Public: {}
Async: {}

Code:
```
{}
```

Provide feedback in JSON format:
{{
    "rating": 0-10,
    "strengths": ["Strength 1", "Strength 2"],
    "weaknesses": ["Weakness 1", "Weakness 2"],
    "suggestions": ["Suggestion 1", "Suggestion 2"],
    "overall_feedback": "Overall feedback text"
}}"#,
            func.name,
            func.file,
            func.params.iter().map(|p| p.as_str()).collect::<Vec<_>>(),
            func.returns,
            func.is_public,
            func.is_async,
            source_preview
        )
    }

    /// Build a prompt for test generation
    pub fn generate_tests(func: &FunctionNode, source_preview: &str) -> String {
        format!(
            r#"Generate unit tests for this function.

Function: {}
File: {}
Parameters: {:?}
Returns: {:?}

Code:
```
{}
```

Generate tests in the same language. Include:
1. Happy path tests
2. Edge case tests
3. Error case tests

Tests:"#,
            func.name,
            func.file,
            func.params.iter().map(|p| p.as_str()).collect::<Vec<_>>(),
            func.returns,
            source_preview
        )
    }

    /// Build a prompt for API documentation
    pub fn generate_api_doc(func: &FunctionNode, source_preview: &str) -> String {
        format!(
            r#"Generate API documentation for this function.

Function: {}
File: {}
Parameters: {:?}
Returns: {:?}
Public: {}
Async: {}

Code:
```
{}
```

Generate documentation with:
1. Description of what it does
2. Parameter descriptions
3. Return value description
4. Example usage
5. Possible errors

API Documentation:"#,
            func.name,
            func.file,
            func.params.iter().map(|p| p.as_str()).collect::<Vec<_>>(),
            func.returns,
            func.is_public,
            func.is_async,
            source_preview
        )
    }

    // ========================================================================
    // Security Analysis Prompts
    // ========================================================================

    /// Build a prompt for security analysis
    pub fn analyze_security(func: &FunctionNode, source_preview: &str) -> String {
        format!(
            r#"Analyze this function for security vulnerabilities.

Function: {}
File: {}
Parameters: {:?}
Returns: {:?}

Code:
```
{}
```

Provide security analysis in JSON format:
{{
    "vulnerabilities": [
        {{
            "severity": "critical|high|medium|low",
            "type": "injection|overflow|auth|access_control|input_validation|crypto|other",
            "description": "What the vulnerability is",
            "location": "Where it occurs",
            "fix": "How to fix it"
        }}
    ],
    "security_score": 0-100,
    "recommendations": ["Recommendation 1", "Recommendation 2"]
}}"#,
            func.name,
            func.file,
            func.params.iter().map(|p| p.as_str()).collect::<Vec<_>>(),
            func.returns,
            source_preview
        )
    }

    // ========================================================================
    // Quick Analysis Prompts (for phi-2)
    // ========================================================================

    /// Quick one-line summary prompt (optimized for phi-2)
    pub fn quick_summary(func_name: &str) -> String {
        format!("Function {} does what? Answer in 5-10 words.", func_name)
    }

    /// Quick duplicate check prompt (optimized for phi-2)
    pub fn quick_duplicate_check(name_a: &str, name_b: &str) -> String {
        format!("Are {} and {} the same? Answer yes/no.", name_a, name_b)
    }

    /// Quick complexity assessment (optimized for phi-2)
    pub fn quick_complexity(func_name: &str) -> String {
        format!("Is {} simple or complex? Answer in one word.", func_name)
    }

    // ========================================================================
    // Error Handling Prompts
    // ========================================================================

    /// Build a prompt for error analysis
    pub fn analyze_errors(func: &FunctionNode, source_preview: &str) -> String {
        format!(
            r#"Analyze error handling in this function.

Function: {}
File: {}
Parameters: {:?}
Returns: {:?}

Code:
```
{}
```

Provide error handling analysis:
1. What errors could occur?
2. Are they properly handled?
3. Suggestions for improvement

Error Analysis:"#,
            func.name,
            func.file,
            func.params.iter().map(|p| p.as_str()).collect::<Vec<_>>(),
            func.returns,
            source_preview
        )
    }

    /// Build a prompt for panic analysis
    pub fn analyze_panics(func: &FunctionNode, source_preview: &str) -> String {
        format!(
            r#"Analyze this function for potential panics.

Function: {}
File: {}
Parameters: {:?}
Returns: {:?}

Code:
```
{}
```

Identify:
1. Where panics could occur
2. Why they could occur
3. How to prevent them

Panic Analysis:"#,
            func.name,
            func.file,
            func.params.iter().map(|p| p.as_str()).collect::<Vec<_>>(),
            func.returns,
            source_preview
        )
    }
}

// ============================================================================
// Prompt Templates (Structured)
// ============================================================================

/// Pre-defined prompt templates with placeholders
pub mod templates {
    /// Template for function analysis
    pub const FUNCTION_ANALYSIS: &str = r#"
Analyze this function:

Function: {name}
File: {file}
Parameters: {params}
Returns: {returns}

Code:
```rust
{code}
```

Provide: summary, complexity, issues, suggestions.
"#;

    /// Template for project overview
    pub const PROJECT_OVERVIEW: &str = r#"
Project Overview:
- Functions: {functions}
- Files: {files}
- Languages: {languages}
- Key components: {components}

Provide a high-level overview.
"#;

    /// Template for code review
    pub const CODE_REVIEW: &str = r#"
Review this code:

File: {file}
Language: {lang}
Code:
```{lang}
{code}
```

Provide: strengths, weaknesses, suggestions.
"#;

    /// Template for documentation generation
    pub const DOCUMENTATION: &str = r#"
Generate documentation for:

File: {file}
Purpose: {purpose}
Key functions: {functions}

Documentation:
"#;
}

// ============================================================================
// Helpers
// ============================================================================

impl PromptBuilder {
    /// Apply a template with variables
    pub fn apply_template(template: &str, variables: &[(&str, &str)]) -> String {
        let mut result = template.to_string();
        for (key, value) in variables {
            result = result.replace(&format!("{{{}}}", key), value);
        }
        result
    }

    /// Build a prompt from a template and a function
    pub fn from_template_with_func(
        template: &str,
        func: &FunctionNode,
        source_preview: &str,
    ) -> String {
        let returns_str = format!("{:?}", func.returns);
        let variables = vec![("returns", returns_str.as_str()), ("code", source_preview)];
        Self::apply_template(template, &variables)
    }
}

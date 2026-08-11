// src/engine/llm_analysis.rs

use crate::graph::call_graph::CallGraph;
use crate::llm::CodeUnderstandingEngine;
use crate::parser::tree_sitter::ParsedFile;
use std::collections::HashMap;

// ⭐ ADD THIS STRUCT
#[derive(Debug, Clone, Default)]
pub struct LLMAnalysis {
    pub has_documentation: bool,
    pub documentation: Option<String>,
    pub function_summaries: Vec<(String, String)>,
    pub issues: Vec<(String, crate::llm::CodeIssue)>,
    pub summarized_count: usize,
    pub issues_count: usize,
}

pub struct LLMAnalyzer;

impl LLMAnalyzer {
    pub async fn analyze(
        engine: &mut CodeUnderstandingEngine,
        call_graph: &CallGraph,
        files: &[ParsedFile],
    ) -> Result<LLMAnalysis, String> {
        let mut analysis = LLMAnalysis::default();

        // Generate documentation
        println!("   📝 Generating documentation...");
        match engine.generate_documentation(call_graph, files).await {
            Ok(doc) => {
                analysis.documentation = Some(doc);
                analysis.has_documentation = true;
            }
            Err(e) => {
                eprintln!("   ❌ Failed to generate documentation: {}", e);
            }
        }

        // Summarize top 3 functions
        let mut important_functions: Vec<_> = call_graph
            .node_indices()
            .map(|idx| (&call_graph[idx], idx))
            .collect();
        important_functions.sort_by(|a, b| b.0.importance_score.total_cmp(&a.0.importance_score));

        let mut source_map = HashMap::new();
        for file in files {
            source_map.insert(file.path.clone(), file.source.clone());
        }

        let mut summaries = Vec::new();
        let mut issues = Vec::new();

        const MAX_FUNCTIONS_TO_ANALYZE: usize = 3;
        const RUN_BUG_ANALYSIS: bool = false;

        for (func, _idx) in important_functions.iter().take(MAX_FUNCTIONS_TO_ANALYZE) {
            if let Some(source) = source_map.get(&func.file) {
                match engine.summarize_function(func, source).await {
                    Ok(summary) => {
                        summaries.push((func.name.clone(), summary));
                        analysis.summarized_count += 1;
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to summarize {}: {}", func.name, e);
                    }
                }

                if RUN_BUG_ANALYSIS {
                    match engine.analyze_bugs(func, source).await {
                        Ok(issues_list) => {
                            for issue in issues_list {
                                issues.push((func.name.clone(), issue));
                            }
                        }
                        Err(e) => {
                            eprintln!("❌ Failed to analyze {}: {}", func.name, e);
                        }
                    }
                }
            }
        }

        analysis.function_summaries = summaries;
        analysis.issues_count = issues.len();
        analysis.issues = issues;
        Ok(analysis)
    }
}

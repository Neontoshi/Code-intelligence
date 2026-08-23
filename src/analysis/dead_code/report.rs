// src/analysis/dead_code/report.rs

use super::analyzer::{ConfidenceLevel, DeadCodeAnalysis};

pub struct DeadCodeReportGenerator;

impl DeadCodeReportGenerator {
    pub fn generate_report(analysis: &DeadCodeAnalysis) -> String {
        let mut output = String::new();

        output.push_str("# 🧹 Dead Code Analysis Report\n\n");

        // Executive Summary
        output.push_str("## 📊 Executive Summary\n\n");
        output.push_str(&format!(
            "- **Total Functions**: {}\n",
            analysis.summary.total_functions
        ));
        output.push_str(&format!(
            "- **Dead Functions**: {} ({:.1}%)\n",
            analysis.summary.dead_functions,
            analysis.summary.dead_functions as f64 / analysis.summary.total_functions as f64
                * 100.0
        ));
        output.push_str(&format!(
            "- **Dead Types**: {}\n",
            analysis.summary.dead_types
        ));
        output.push_str(&format!(
            "- **Dead Modules**: {}\n",
            analysis.summary.dead_modules
        ));
        output.push_str(&format!(
            "- **Dead Files**: {}\n",
            analysis.summary.dead_files
        ));
        output.push_str(&format!(
            "- **Average Confidence**: {:.1}%\n",
            analysis.summary.avg_confidence * 100.0
        ));
        output.push_str(&format!(
            "- **Estimated LOC Removable**: {}\n\n",
            analysis.summary.estimated_loc_removable
        ));

        // Priority Report
        output.push_str("## 🎯 Priority Removal Order\n\n");
        output.push_str("| # | Function | Confidence | Impact | LOC |\n");
        output.push_str("|---|----------|------------|--------|-----|\n");

        for func in &analysis.functions {
            let confidence_str = match func.score.level {
                ConfidenceLevel::Guaranteed => "🔴 95-100%",
                ConfidenceLevel::VeryLikely => "🟠 80-95%",
                ConfidenceLevel::Probably => "🟡 60-80%",
                _ => "🟢 40-60%",
            };

            output.push_str(&format!(
                "| {} | `{}` | {} | {} | {} |\n",
                func.removal_order,
                func.name,
                confidence_str,
                func.impact.estimated_removal_impact,
                func.impact.lines_of_code
            ));
        }
        output.push('\n');

        // Detailed Functions
        output.push_str("## 🔍 Detailed Dead Function Analysis\n\n");

        for func in &analysis.functions {
            output.push_str(&format!("### {}. `{}`\n\n", func.removal_order, func.name));
            output.push_str(&format!(
                "- **File**: `{}` (line {})\n",
                func.file, func.line
            ));
            output.push_str(&format!(
                "- **Confidence**: {:.1}%\n",
                func.score.score * 100.0
            ));
            output.push_str(&format!("- **Level**: {:?}\n", func.score.level));
            output.push_str(&format!(
                "- **Complexity**: {:.2}\n",
                func.impact.complexity
            ));
            output.push_str(&format!(
                "- **Estimated LOC**: {}\n",
                func.impact.lines_of_code
            ));
            output.push_str(&format!(
                "- **Dependencies**: {}\n",
                func.impact.dependencies.len()
            ));
            output.push_str(&format!(
                "- **Impact**: {}\n",
                func.impact.estimated_removal_impact
            ));

            output.push_str("\n**Factors:**\n");
            for factor in &func.score.factors {
                let sign = if factor.contribution > 0.0 { "+" } else { "" };
                output.push_str(&format!(
                    "  - {}: {}{:.1}\n",
                    factor.name, sign, factor.contribution
                ));
            }
            output.push('\n');
        }

        // Dead Types
        if !analysis.types.unused_structs.is_empty() {
            output.push_str("## 🏗️ Dead Types\n\n");
            output.push_str("### Unused Structs\n\n");
            for t in &analysis.types.unused_structs {
                output.push_str(&format!("- `{}` ({}) - {}\n", t.name, t.file, t.reason));
            }
            output.push('\n');
        }

        // Dead Modules
        if !analysis.modules.unused_modules.is_empty() {
            output.push_str("## 📁 Dead Modules\n\n");
            for m in &analysis.modules.unused_modules {
                output.push_str(&format!(
                    "- `{}` - {} (confidence: {:.1}%)\n",
                    m.name,
                    m.reason,
                    m.confidence * 100.0
                ));
            }
            output.push('\n');
        }

        // Recommendations
        output.push_str("## 💡 Recommendations\n\n");

        if analysis.summary.dead_functions > 10 {
            output.push_str("1. **Start with high-confidence functions** - Remove functions with Guaranteed/VeryLikely confidence first\n");
        }

        if analysis.summary.dead_types > 0 {
            output.push_str("2. **Review dead types** - Unused structs and enums may indicate dead code paths\n");
        }

        if analysis.summary.dead_modules > 0 {
            output.push_str(
                "3. **Consider module removal** - Dead modules can be removed entirely\n",
            );
        }

        if analysis.summary.estimated_loc_removable > 1000 {
            output.push_str(
                "4. **Significant LOC reduction possible** - Consider prioritizing this cleanup\n",
            );
        }

        output.push_str("\n---\n");
        output.push_str("*Report generated by Code Intelligence Dead Code Analyzer*\n");

        output
    }
}

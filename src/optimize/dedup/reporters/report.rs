use crate::optimize::dedup::types::DeduplicationResult;
use crate::optimize::dedup::DuplicateType;

pub struct ReportGenerator;

impl ReportGenerator {
    pub fn generate(result: &DeduplicationResult) -> String {
        let mut output = String::new();

        output.push_str("# 🔍 Deduplication Report\n\n");

        output.push_str("## 📊 Summary\n\n");
        output.push_str(&format!(
            "- **Duplicate groups found**: {}\n",
            result.duplicate_groups.len()
        ));
        output.push_str(&format!(
            "- **Total token savings**: ~{}\n",
            result.total_saved_tokens
        ));
        output.push_str(&format!(
            "- **Unique functions**: {}\n",
            result.unique_functions.len()
        ));
        output.push_str(&format!(
            "- **Confidence score**: {:.1}%\n\n",
            result.accuracy_metrics.confidence_score * 100.0
        ));

        output.push_str("### 📈 Accuracy Metrics\n\n");
        output.push_str(&format!(
            "- Exact matches: {}\n",
            result.accuracy_metrics.exact_matches
        ));
        output.push_str(&format!(
            "- Structural matches: {}\n",
            result.accuracy_metrics.structural_matches
        ));
        output.push_str(&format!(
            "- Algorithmic matches: {}\n",
            result.accuracy_metrics.algorithmic_matches
        ));
        output.push_str(&format!(
            "- False positives filtered: {}\n\n",
            result.accuracy_metrics.false_positives_filtered
        ));

        if result.duplicate_groups.is_empty() {
            output.push_str("✅ **No duplicate code found!** Great job!\n\n");
        } else {
            let mut groups = result.duplicate_groups.clone();
            groups.sort_by(|a, b| {
                b.priority_score
                    .partial_cmp(&a.priority_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| {
                        let a_key = a
                            .functions
                            .first()
                            .map(|f| f.full_path.as_str())
                            .unwrap_or("");
                        let b_key = b
                            .functions
                            .first()
                            .map(|f| f.full_path.as_str())
                            .unwrap_or("");
                        a_key.cmp(b_key)
                    })
            });

            output.push_str("## 🎯 Duplicate Groups (by priority)\n\n");
            output.push_str(
                "> Groups are prioritized by: group size, complexity, and call frequency.\n\n",
            );

            // Summary table
            output.push_str("| Priority | Type | Functions | Similarity | Token Savings |\n");
            output.push_str("|----------|------|-----------|------------|---------------|\n");

            for (i, group) in groups.iter().enumerate() {
                let priority_emoji = if group.priority_score > 0.8 {
                    "🔥"
                } else if group.priority_score > 0.5 {
                    "⚠️"
                } else {
                    "ℹ️"
                };
                output.push_str(&format!(
                    "| {} {} | {:?} | {} | {:.1}% | ~{} |\n",
                    priority_emoji,
                    i + 1,
                    group.duplicate_type,
                    group.functions.len(),
                    group.similarity_score * 100.0,
                    group.total_token_savings
                ));
            }
            output.push('\n');

            // Detailed groups
            output.push_str("### 📝 Detailed Groups\n\n");

            for (i, group) in groups.iter().enumerate() {
                let priority_emoji = if group.priority_score > 0.8 {
                    "🔥 HIGH"
                } else if group.priority_score > 0.5 {
                    "⚠️ MEDIUM"
                } else {
                    "ℹ️ LOW"
                };

                output.push_str(&format!(
                    "### Group {} ({} functions) - Priority: {}\n\n",
                    i + 1,
                    group.functions.len(),
                    priority_emoji
                ));

                output.push_str(&format!("**Type**: {:?}\n", group.duplicate_type));
                output.push_str(&format!(
                    "**Similarity**: {:.1}%\n",
                    group.similarity_score * 100.0
                ));
                output.push_str(&format!(
                    "**Suggested**: {}\n",
                    group.refactoring_suggestion
                ));
                output.push_str(&format!(
                    "**Estimated savings**: ~{} tokens\n",
                    group.estimated_savings
                ));
                output.push_str(&format!(
                    "**Token savings**: ~{} tokens\n",
                    group.total_token_savings
                ));
                output.push_str(&format!(
                    "**Complexity impact**: {:.2}\n\n",
                    group.complexity_impact
                ));

                output.push_str("**Functions:**\n\n");
                for func in &group.functions {
                    let call_count = func.fan_in;
                    let layer = if func.layer.is_empty() {
                        "unknown"
                    } else {
                        &func.layer
                    };
                    output.push_str(&format!(
                        "- `{}` ({}:{}) [calls: {}, layer: {}]\n",
                        func.name,
                        func.file.split('/').last().unwrap_or(&func.file),
                        func.line,
                        call_count,
                        layer
                    ));
                }
                output.push('\n');
            }
        }

        output.push_str("## 💡 Recommendations\n\n");

        let exact_count = result
            .duplicate_groups
            .iter()
            .filter(|g| matches!(g.duplicate_type, DuplicateType::Exact))
            .count();

        let structural_count = result
            .duplicate_groups
            .iter()
            .filter(|g| matches!(g.duplicate_type, DuplicateType::Structural))
            .count();

        let high_priority_count = result
            .duplicate_groups
            .iter()
            .filter(|g| g.priority_score > 0.8)
            .count();

        if high_priority_count > 0 {
            output.push_str(&format!("1. **🔥 High-priority duplicates** - {} groups with high impact. Address these first.\n", high_priority_count));
        }
        if exact_count > 0 {
            output.push_str("2. **Extract exact duplicates** - These functions are identical and should be unified\n");
        }
        if structural_count > 0 {
            output.push_str("3. **Refactor structural duplicates** - These share similar patterns; consider extracting a shared function\n");
        }
        if !result.duplicate_groups.is_empty() {
            output.push_str("4. **Review duplicate groups** - Not all duplicates need to be removed, but they should be justified\n");
        }
        output.push_str(
            "5. **Run tests after refactoring** - Ensure changes don't break functionality\n",
        );

        output
    }
}

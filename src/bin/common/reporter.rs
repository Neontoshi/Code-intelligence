// src/bin/common/reporter.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorReport {
    pub timestamp: u64,
    pub error_type: String,
    pub message: String,
    pub severity: String,
    pub context: HashMap<String, String>,
    pub stack: Vec<String>,
    pub user_action: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionReport {
    pub tool: String,
    pub version: String,
    pub environment: String,
    pub errors: Vec<ErrorReport>,
    pub warnings: Vec<ErrorReport>,
    pub metrics: HashMap<String, String>,
}

pub struct Reporter {
    reports: Vec<ErrorReport>,
    warnings: Vec<ErrorReport>,
    metrics: HashMap<String, String>,
    tool_name: String,
    version: String,
    environment: String,
}

impl Reporter {
    pub fn new(tool_name: &str, version: &str, environment: &str) -> Self {
        Self {
            reports: Vec::new(),
            warnings: Vec::new(),
            metrics: HashMap::new(),
            tool_name: tool_name.to_string(),
            version: version.to_string(),
            environment: environment.to_string(),
        }
    }

    pub fn report_error(
        &mut self,
        error_type: &str,
        message: &str,
        severity: &str,
        context: HashMap<String, String>,
    ) {
        let report = ErrorReport {
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            error_type: error_type.to_string(),
            message: message.to_string(),
            severity: severity.to_string(),
            context,
            stack: Vec::new(),
            user_action: None,
        };
        self.reports.push(report);
    }

    pub fn report_warning(
        &mut self,
        error_type: &str,
        message: &str,
        context: HashMap<String, String>,
    ) {
        let report = ErrorReport {
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            error_type: error_type.to_string(),
            message: message.to_string(),
            severity: "warning".to_string(),
            context,
            stack: Vec::new(),
            user_action: None,
        };
        self.warnings.push(report);
    }

    pub fn set_metric(&mut self, key: &str, value: &str) {
        self.metrics.insert(key.to_string(), value.to_string());
    }

    pub fn generate_report(&self) -> ProductionReport {
        ProductionReport {
            tool: self.tool_name.clone(),
            version: self.version.clone(),
            environment: self.environment.clone(),
            errors: self.reports.clone(),
            warnings: self.warnings.clone(),
            metrics: self.metrics.clone(),
        }
    }

    pub fn print_report(&self) {
        let report = self.generate_report();
        println!("\n📊 Production Report");
        println!("===================");
        println!("Tool: {}", report.tool);
        println!("Version: {}", report.version);
        println!("Environment: {}", report.environment);

        if !report.errors.is_empty() {
            println!("\n❌ Errors ({}):", report.errors.len());
            for error in &report.errors {
                println!(
                    "  - [{}] {}: {}",
                    error.severity, error.error_type, error.message
                );
            }
        }

        if !report.warnings.is_empty() {
            println!("\n⚠️ Warnings ({}):", report.warnings.len());
            for warning in &report.warnings {
                println!("  - {}: {}", warning.error_type, warning.message);
            }
        }

        if !report.metrics.is_empty() {
            println!("\n📈 Metrics:");
            for (key, value) in &report.metrics {
                println!("  - {}: {}", key, value);
            }
        }
    }

    pub fn to_json(&self) -> Result<String, String> {
        let report = self.generate_report();
        serde_json::to_string_pretty(&report)
            .map_err(|e| format!("Failed to serialize report: {}", e))
    }

    pub fn to_markdown(&self) -> String {
        let report = self.generate_report();
        let mut md = String::new();

        md.push_str(&format!("# Production Report\n\n"));
        md.push_str(&format!("**Tool**: {}\n", report.tool));
        md.push_str(&format!("**Version**: {}\n", report.version));
        md.push_str(&format!("**Environment**: {}\n\n", report.environment));

        if !report.errors.is_empty() {
            md.push_str("## ❌ Errors\n\n");
            for error in &report.errors {
                md.push_str(&format!("- **{}**: {}\n", error.error_type, error.message));
                if !error.context.is_empty() {
                    md.push_str("  - Context:\n");
                    for (k, v) in &error.context {
                        md.push_str(&format!("    - {}: {}\n", k, v));
                    }
                }
            }
            md.push('\n');
        }

        if !report.warnings.is_empty() {
            md.push_str("## ⚠️ Warnings\n\n");
            for warning in &report.warnings {
                md.push_str(&format!(
                    "- **{}**: {}\n",
                    warning.error_type, warning.message
                ));
            }
            md.push('\n');
        }

        if !report.metrics.is_empty() {
            md.push_str("## 📈 Metrics\n\n");
            md.push_str("| Metric | Value |\n");
            md.push_str("|--------|-------|\n");
            for (key, value) in &report.metrics {
                md.push_str(&format!("| {} | {} |\n", key, value));
            }
        }

        md
    }
}

impl Default for Reporter {
    fn default() -> Self {
        Self::new("code-intelligence", env!("CARGO_PKG_VERSION"), "production")
    }
}

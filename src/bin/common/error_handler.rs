// src/bin/common/error_handler.rs

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    User,
    Config,
    System,
    Internal,
}

/// Production error handler
pub struct ErrorHandler {
    verbose: bool,
    json_logging: bool,
}

impl ErrorHandler {
    pub fn new(verbose: bool, json_logging: bool) -> Self {
        Self {
            verbose,
            json_logging,
        }
    }

    /// Handle an error with proper formatting
    pub fn handle_error(&self, error: &dyn std::error::Error, severity: ErrorSeverity) -> ! {
        if self.json_logging {
            self.log_error_json(error, severity);
        } else {
            self.log_error_human(error, severity);
        }

        let code = match severity {
            ErrorSeverity::User => 1,
            ErrorSeverity::Config => 2,
            ErrorSeverity::System => 3,
            ErrorSeverity::Internal => 4,
        };
        std::process::exit(code);
    }

    /// Handle an error with additional context
    pub fn handle_error_with_context(
        &self,
        error: &dyn std::error::Error,
        severity: ErrorSeverity,
        context: &str,
    ) -> ! {
        if self.json_logging {
            self.log_error_json_with_context(error, severity, context);
        } else {
            self.log_error_human_with_context(error, severity, context);
        }

        let code = match severity {
            ErrorSeverity::User => 1,
            ErrorSeverity::Config => 2,
            ErrorSeverity::System => 3,
            ErrorSeverity::Internal => 4,
        };
        std::process::exit(code);
    }

    fn log_error_human(&self, error: &dyn std::error::Error, severity: ErrorSeverity) {
        let prefix = match severity {
            ErrorSeverity::User => "❌ Error",
            ErrorSeverity::Config => "⚙️ Configuration Error",
            ErrorSeverity::System => "⚠️ System Error",
            ErrorSeverity::Internal => "🐛 Internal Error",
        };

        eprintln!("{}: {}", prefix, error);

        if self.verbose {
            eprintln!("\n📋 Details:");
            if let Some(source) = error.source() {
                eprintln!("   Caused by: {}", source);
            }
            eprintln!("   Severity: {:?}", severity);
        }
    }

    fn log_error_human_with_context(
        &self,
        error: &dyn std::error::Error,
        severity: ErrorSeverity,
        context: &str,
    ) {
        self.log_error_human(error, severity);
        eprintln!("   Context: {}", context);
    }

    fn log_error_json(&self, error: &dyn std::error::Error, severity: ErrorSeverity) {
        use serde_json::json;

        let entry = json!({
            "level": "error",
            "severity": format!("{:?}", severity),
            "message": error.to_string(),
            "source": error.source().map(|s| s.to_string()),
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });

        eprintln!(
            "{}",
            serde_json::to_string_pretty(&entry).unwrap_or_default()
        );
    }

    fn log_error_json_with_context(
        &self,
        error: &dyn std::error::Error,
        severity: ErrorSeverity,
        context: &str,
    ) {
        use serde_json::json;

        let entry = json!({
            "level": "error",
            "severity": format!("{:?}", severity),
            "message": error.to_string(),
            "context": context,
            "source": error.source().map(|s| s.to_string()),
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });

        eprintln!(
            "{}",
            serde_json::to_string_pretty(&entry).unwrap_or_default()
        );
    }
}

impl Default for ErrorHandler {
    fn default() -> Self {
        Self::new(false, false)
    }
}

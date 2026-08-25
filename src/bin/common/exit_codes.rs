// src/bin/common/exit_codes.rs

/// Exit codes for the code-intelligence CLI
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ExitCode {
    /// Success - everything worked
    Success = 0,

    /// Analysis found issues (dead code, duplicates, etc.)
    AnalysisFoundIssues = 1,

    /// Configuration error
    ConfigError = 2,

    /// Model error (not found, corrupt, etc.)
    ModelError = 3,

    /// Repository error (invalid path, not a repo, etc.)
    RepositoryError = 4,

    /// Internal failure (unexpected error)
    InternalFailure = 5,

    /// Parse error (invalid input, malformed file, etc.)
    ParseError = 6,

    /// IO error (permission denied, file not found, etc.)
    IoError = 7,

    /// Timeout
    Timeout = 8,

    /// Cancelled by user
    Cancelled = 130,

    /// Unknown error
    Unknown = 255,
}

impl ExitCode {
    #[allow(dead_code)]
    pub fn as_i32(&self) -> i32 {
        *self as i32
    }

    #[allow(dead_code)]
    pub fn exit(&self) -> ! {
        std::process::exit(self.as_i32())
    }
}

/// Convert any error to an appropriate exit code
#[allow(dead_code)]
pub fn error_to_exit_code(error: &dyn std::error::Error) -> ExitCode {
    let error_str = error.to_string();

    if error_str.contains("config") || error_str.contains("Config") {
        ExitCode::ConfigError
    } else if error_str.contains("model") || error_str.contains("Model") {
        ExitCode::ModelError
    } else if error_str.contains("parse") || error_str.contains("Parse") {
        ExitCode::ParseError
    } else if error_str.contains("permission")
        || error_str.contains("Permission")
        || error_str.contains("denied")
    {
        ExitCode::IoError
    } else if error_str.contains("not found") || error_str.contains("Not found") {
        ExitCode::RepositoryError
    } else if error_str.contains("timeout") || error_str.contains("Timeout") {
        ExitCode::Timeout
    } else if error_str.contains("cancelled") || error_str.contains("Cancelled") {
        ExitCode::Cancelled
    } else {
        ExitCode::InternalFailure
    }
}

/// Set the exit code for a Result
#[allow(dead_code)]
pub fn exit_with_result<T, E>(result: Result<T, E>) -> T
where
    E: std::error::Error + 'static,
{
    match result {
        Ok(value) => value,
        Err(err) => {
            let exit_code = error_to_exit_code(&err);
            eprintln!("❌ Error: {}", err);
            exit_code.exit();
        }
    }
}

/// Exit with a specific message and code
#[allow(dead_code)]
pub fn exit_with_message(message: &str, code: ExitCode) -> ! {
    eprintln!("{}", message);
    code.exit()
}

/// Exit with success
#[allow(dead_code)]
pub fn exit_success() -> ! {
    ExitCode::Success.exit()
}

/// Exit with failure
#[allow(dead_code)]
pub fn exit_failure(message: &str) -> ! {
    exit_with_message(message, ExitCode::InternalFailure)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exit_codes() {
        assert_eq!(ExitCode::Success.as_i32(), 0);
        assert_eq!(ExitCode::ConfigError.as_i32(), 2);
        assert_eq!(ExitCode::ModelError.as_i32(), 3);
        assert_eq!(ExitCode::InternalFailure.as_i32(), 5);
    }

    #[test]
    fn test_error_to_exit_code() {
        use std::io;

        let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "permission denied");
        assert!(matches!(error_to_exit_code(&io_error), ExitCode::IoError));

        let config_error = std::io::Error::new(std::io::ErrorKind::Other, "config error");
        assert!(matches!(
            error_to_exit_code(&config_error),
            ExitCode::ConfigError
        ));
    }
}

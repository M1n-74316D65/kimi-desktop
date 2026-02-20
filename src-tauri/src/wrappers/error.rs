//! Error types for wrapper operations

use std::fmt;

/// Errors that can occur during wrapper operations
#[derive(Debug, Clone)]
pub enum WrapperError {
    /// Failed to execute JavaScript in the webview
    InjectionFailed { context: String, source: String },

    /// Target window not found
    WindowNotFound { window_label: String },

    /// JavaScript evaluation error
    EvalError { message: String },

    /// JSON serialization/deserialization error
    SerializationError { message: String },

    /// Operation timed out
    Timeout { operation: String, duration_ms: u64 },

    /// Invalid URL provided
    InvalidUrl { url: String, reason: String },

    /// Store operation failed
    StoreError { operation: String, message: String },

    /// Settings validation failed
    InvalidSettings { field: String, message: String },
}

impl fmt::Display for WrapperError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WrapperError::InjectionFailed { context, source } => {
                write!(f, "Failed to inject {}: {}", context, source)
            }
            WrapperError::WindowNotFound { window_label } => {
                write!(f, "Window '{}' not found", window_label)
            }
            WrapperError::EvalError { message } => {
                write!(f, "JavaScript evaluation error: {}", message)
            }
            WrapperError::SerializationError { message } => {
                write!(f, "Serialization error: {}", message)
            }
            WrapperError::Timeout {
                operation,
                duration_ms,
            } => {
                write!(
                    f,
                    "Operation '{}' timed out after {}ms",
                    operation, duration_ms
                )
            }
            WrapperError::InvalidUrl { url, reason } => {
                write!(f, "Invalid URL '{}': {}", url, reason)
            }
            WrapperError::StoreError { operation, message } => {
                write!(f, "Store operation '{}' failed: {}", operation, message)
            }
            WrapperError::InvalidSettings { field, message } => {
                write!(f, "Invalid settings field '{}': {}", field, message)
            }
        }
    }
}

impl std::error::Error for WrapperError {}

impl From<serde_json::Error> for WrapperError {
    fn from(err: serde_json::Error) -> Self {
        WrapperError::SerializationError {
            message: err.to_string(),
        }
    }
}

impl From<tauri::Error> for WrapperError {
    fn from(err: tauri::Error) -> Self {
        WrapperError::EvalError {
            message: err.to_string(),
        }
    }
}

/// Result type for wrapper operations
pub type WrapperResult<T> = Result<T, WrapperError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display_injection_failed() {
        let err = WrapperError::InjectionFailed {
            context: "message".to_string(),
            source: "textarea not found".to_string(),
        };
        assert!(err.to_string().contains("Failed to inject message"));
        assert!(err.to_string().contains("textarea not found"));
    }

    #[test]
    fn test_error_display_window_not_found() {
        let err = WrapperError::WindowNotFound {
            window_label: "main".to_string(),
        };
        assert!(err.to_string().contains("Window 'main' not found"));
    }

    #[test]
    fn test_from_serde_error() {
        let json = "{ invalid json }";
        let result: Result<serde_json::Value, _> = serde_json::from_str(json);
        let err: WrapperError = result.unwrap_err().into();

        match err {
            WrapperError::SerializationError { .. } => {}
            _ => panic!("Expected SerializationError"),
        }
    }

    #[test]
    fn test_error_debug_format() {
        let err = WrapperError::Timeout {
            operation: "message injection".to_string(),
            duration_ms: 5000,
        };
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("Timeout"));
    }
}

use std::convert::From;
use thiserror::Error;

/// Error types for the structured logger
#[derive(Error, Debug)]
pub enum LoggerError {
    /// Error setting the logger
    #[error("Failed to set logger")]
    SetLoggerError,

    /// Error writing to output
    #[error("Failed to write log: {0}")]
    IoError(#[from] std::io::Error),

    /// Error serializing log entry
    #[error("Failed to serialize log entry: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// Other errors
    #[error("Logger error: {0}")]
    Other(String),
}

// Manually implement From<log::SetLoggerError> for LoggerError
impl From<log::SetLoggerError> for LoggerError {
    fn from(_: log::SetLoggerError) -> Self {
        LoggerError::SetLoggerError
    }
}

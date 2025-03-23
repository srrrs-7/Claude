use thiserror::Error;

/// Error types for the structured logger
#[derive(Error, Debug)]
pub enum LoggerError {
    /// Error setting the logger
    #[error("Failed to set logger: {0}")]
    SetLoggerError(#[from] log::SetLoggerError),
    
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

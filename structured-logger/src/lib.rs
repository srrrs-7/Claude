pub mod config;
pub mod error;
pub mod formatter;
pub mod logger;

use crate::config::LoggerConfig;
use crate::logger::StructuredLogger;
use log::LevelFilter;

/// Initialize the structured logger with default configuration
pub fn init() -> Result<(), error::LoggerError> {
    let config = LoggerConfig::default();
    init_with_config(config)
}

/// Initialize the structured logger with custom configuration
pub fn init_with_config(config: LoggerConfig) -> Result<(), error::LoggerError> {
    let max_level = config.level;
    let logger = StructuredLogger::new(config);
    
    log::set_boxed_logger(Box::new(logger))?;
    log::set_max_level(max_level);
    
    Ok(())
}

/// A module that re-exports the log macros for convenience
pub mod macros {
    pub use log::{debug, error, info, log, log_enabled, trace, warn};
}

// Re-export the log crate's LevelFilter for convenience
pub use log::LevelFilter;

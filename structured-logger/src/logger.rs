use crate::config::{LoggerConfig, OutputFormat};
use crate::error::LoggerError;
use crate::formatter::LogFormatter;
use log::{LevelFilter, Metadata, Record};
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::sync::Mutex;

/// A structured logger implementation
pub struct StructuredLogger {
    formatter: LogFormatter,
    config: LoggerConfig,
    output: Mutex<Box<dyn Write + Send>>,
}

impl StructuredLogger {
    /// Create a new structured logger
    pub fn new(config: LoggerConfig) -> Self {
        let formatter = LogFormatter::new(
            config.include_file_info,
            config.include_module_path,
            config.include_timestamp,
            config.timestamp_format.clone(),
            config.global_metadata.clone(),
            config.format,
        );
        
        let output: Box<dyn Write + Send> = match &config.output_path {
            Some(path) => {
                let file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)
                    .unwrap_or_else(|_| panic!("Failed to open log file: {}", path));
                
                Box::new(file)
            }
            None => Box::new(io::stdout()),
        };
        
        Self {
            formatter,
            config,
            output: Mutex::new(output),
        }
    }
    
    /// Create a builder for custom logger configuration
    pub fn builder() -> LoggerConfig {
        LoggerConfig::new()
    }
}

impl log::Log for StructuredLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.config.level
    }
    
    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            match self.formatter.format(record) {
                Ok(formatted) => {
                    let mut output = self.output.lock().unwrap();
                    let _ = writeln!(output, "{}", formatted);
                    let _ = output.flush();
                }
                Err(err) => {
                    eprintln!("Error formatting log: {}", err);
                }
            }
        }
    }
    
    fn flush(&self) {
        let _ = self.output.lock().unwrap().flush();
    }
}

/// A context builder for structured logging with additional metadata
pub struct LogContext {
    metadata: std::collections::HashMap<String, serde_json::Value>,
}

impl LogContext {
    /// Create a new log context
    pub fn new() -> Self {
        Self {
            metadata: std::collections::HashMap::new(),
        }
    }
    
    /// Add a string value to the context
    pub fn with_str(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), serde_json::Value::String(value.to_string()));
        self
    }
    
    /// Add a numeric value to the context
    pub fn with_number<T: Into<f64>>(mut self, key: &str, value: T) -> Self {
        if let Some(num) = serde_json::Number::from_f64(value.into()) {
            self.metadata.insert(key.to_string(), serde_json::Value::Number(num));
        }
        self
    }
    
    /// Add a boolean value to the context
    pub fn with_bool(mut self, key: &str, value: bool) -> Self {
        self.metadata.insert(key.to_string(), serde_json::Value::Bool(value));
        self
    }
    
    /// Log at info level with this context
    pub fn info(&self, message: &str) {
        let metadata_str = self.format_metadata();
        log::info!("{} {}", message, metadata_str);
    }
    
    /// Log at error level with this context
    pub fn error(&self, message: &str) {
        let metadata_str = self.format_metadata();
        log::error!("{} {}", message, metadata_str);
    }
    
    /// Log at warn level with this context
    pub fn warn(&self, message: &str) {
        let metadata_str = self.format_metadata();
        log::warn!("{} {}", message, metadata_str);
    }
    
    /// Log at debug level with this context
    pub fn debug(&self, message: &str) {
        let metadata_str = self.format_metadata();
        log::debug!("{} {}", message, metadata_str);
    }
    
    /// Log at trace level with this context
    pub fn trace(&self, message: &str) {
        let metadata_str = self.format_metadata();
        log::trace!("{} {}", message, metadata_str);
    }
    
    /// Format metadata as key=value pairs
    fn format_metadata(&self) -> String {
        self.metadata
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

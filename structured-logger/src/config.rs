use log::LevelFilter;
use std::collections::HashMap;

/// Logger output format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// JSON format for machine processing
    Json,
    /// Pretty format for human readability (colored in terminal)
    Pretty,
    /// Compact format (single line, minimal)
    Compact,
}

impl Default for OutputFormat {
    fn default() -> Self {
        OutputFormat::Json
    }
}

/// Configuration for the structured logger
#[derive(Debug, Clone)]
pub struct LoggerConfig {
    /// Log level filter
    pub level: LevelFilter,
    /// Output format
    pub format: OutputFormat,
    /// Global metadata to include in every log entry
    pub global_metadata: HashMap<String, String>,
    /// Whether to include file and line information
    pub include_file_info: bool,
    /// Whether to include module path
    pub include_module_path: bool,
    /// Whether to include timestamps
    pub include_timestamp: bool,
    /// Custom timestamp format (RFC3339 if None)
    pub timestamp_format: Option<String>,
    /// Output destination (stdout if None)
    pub output_path: Option<String>,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            level: LevelFilter::Info,
            format: OutputFormat::default(),
            global_metadata: HashMap::new(),
            include_file_info: true,
            include_module_path: true,
            include_timestamp: true,
            timestamp_format: None,
            output_path: None,
        }
    }
}

impl LoggerConfig {
    /// Create a new configuration with sensible defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the log level
    pub fn with_level(mut self, level: LevelFilter) -> Self {
        self.level = level;
        self
    }

    /// Set the output format
    pub fn with_format(mut self, format: OutputFormat) -> Self {
        self.format = format;
        self
    }

    /// Add a global metadata field that will be included in all log entries
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.global_metadata
            .insert(key.to_string(), value.to_string());
        self
    }

    /// Enable or disable file information (file name and line number)
    pub fn with_file_info(mut self, enabled: bool) -> Self {
        self.include_file_info = enabled;
        self
    }

    /// Enable or disable module path information
    pub fn with_module_path(mut self, enabled: bool) -> Self {
        self.include_module_path = enabled;
        self
    }

    /// Enable or disable timestamps
    pub fn with_timestamp(mut self, enabled: bool) -> Self {
        self.include_timestamp = enabled;
        self
    }

    /// Set a custom timestamp format
    pub fn with_timestamp_format(mut self, format: &str) -> Self {
        self.timestamp_format = Some(format.to_string());
        self
    }

    /// Set an output path (file) instead of stdout
    pub fn with_output_path(mut self, path: &str) -> Self {
        self.output_path = Some(path.to_string());
        self
    }
}

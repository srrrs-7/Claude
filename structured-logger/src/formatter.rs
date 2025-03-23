use crate::config::OutputFormat;
use crate::error::LoggerError;
use chrono::{Local, Utc};
use colored::{Color, Colorize};
use log::Record;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// A structured log entry
#[derive(Serialize, Deserialize, Debug)]
pub struct LogEntry {
    /// Timestamp of the log entry
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,

    /// Log level (INFO, ERROR, etc.)
    pub level: String,

    /// Log message
    pub message: String,

    /// Source file and line number
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,

    /// Module path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module_path: Option<String>,

    /// Additional context data
    #[serde(flatten)]
    pub metadata: HashMap<String, Value>,
}

/// Formatter for log entries
pub struct LogFormatter {
    include_file_info: bool,
    include_module_path: bool,
    include_timestamp: bool,
    timestamp_format: Option<String>,
    global_metadata: HashMap<String, String>,
    format: OutputFormat,
}

impl LogFormatter {
    /// Create a new formatter
    pub fn new(
        include_file_info: bool,
        include_module_path: bool,
        include_timestamp: bool,
        timestamp_format: Option<String>,
        global_metadata: HashMap<String, String>,
        format: OutputFormat,
    ) -> Self {
        Self {
            include_file_info,
            include_module_path,
            include_timestamp,
            timestamp_format,
            global_metadata,
            format,
        }
    }

    /// Format a log record
    pub fn format(&self, record: &Record) -> Result<String, LoggerError> {
        match self.format {
            OutputFormat::Json => self.format_json(record),
            OutputFormat::Pretty => self.format_pretty(record),
            OutputFormat::Compact => self.format_compact(record),
        }
    }

    /// Format as JSON
    fn format_json(&self, record: &Record) -> Result<String, LoggerError> {
        let entry = self.create_log_entry(record);
        let json = serde_json::to_string(&entry)?;
        Ok(json)
    }

    /// Format for human readability with colors
    fn format_pretty(&self, record: &Record) -> Result<String, LoggerError> {
        let entry = self.create_log_entry(record);

        let level_color = match record.level() {
            log::Level::Error => Color::Red,
            log::Level::Warn => Color::Yellow,
            log::Level::Info => Color::Green,
            log::Level::Debug => Color::Blue,
            log::Level::Trace => Color::Magenta,
        };

        let level_str = format!("[{}]", entry.level).color(level_color).bold();

        let mut output = String::new();

        // Add timestamp if enabled
        if let Some(timestamp) = entry.timestamp {
            output.push_str(&format!("{} ", timestamp.dimmed()));
        }

        // Add level and message
        output.push_str(&format!("{} {}", level_str, entry.message));

        // Add location if available
        if let Some(location) = entry.location {
            output.push_str(&format!(" {}", location.dimmed()));
        }

        // Add module path if available
        if let Some(module_path) = entry.module_path {
            output.push_str(&format!(" [{}]", module_path.dimmed()));
        }

        // Add metadata
        if !entry.metadata.is_empty() {
            output.push_str(" {");
            let metadata_str = entry
                .metadata
                .iter()
                .map(|(k, v)| format!("{}={}", k.cyan(), format!("{}", v).yellow()))
                .collect::<Vec<_>>()
                .join(", ");
            output.push_str(&metadata_str);
            output.push('}');
        }

        Ok(output)
    }

    /// Format as compact single line
    fn format_compact(&self, record: &Record) -> Result<String, LoggerError> {
        let entry = self.create_log_entry(record);

        let mut parts = Vec::new();

        // Add timestamp if enabled
        if let Some(timestamp) = entry.timestamp {
            parts.push(timestamp);
        }

        // Add level and message
        parts.push(format!("[{}] {}", entry.level, entry.message));

        // Add metadata if not empty
        if !entry.metadata.is_empty() {
            let metadata_str = entry
                .metadata
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join(",");
            parts.push(metadata_str);
        }

        Ok(parts.join(" "))
    }

    /// Create a structured log entry from a record
    fn create_log_entry(&self, record: &Record) -> LogEntry {
        let timestamp = if self.include_timestamp {
            Some(self.format_timestamp())
        } else {
            None
        };

        let location = if self.include_file_info {
            match (record.file(), record.line()) {
                (Some(file), Some(line)) => Some(format!("{}:{}", file, line)),
                _ => None,
            }
        } else {
            None
        };

        let module_path = if self.include_module_path {
            record.module_path().map(|s| s.to_string())
        } else {
            None
        };

        // Convert global metadata to serde_json::Value
        let mut metadata = HashMap::new();
        for (key, value) in &self.global_metadata {
            metadata.insert(key.clone(), Value::String(value.clone()));
        }

        // Extract any key-value pairs from the message using the format: "message key1=value1 key2=value2"
        let message = record.args().to_string();
        let (message, kv_pairs) = self.extract_key_values(&message);

        // Add extracted key-value pairs to metadata
        for (key, value) in kv_pairs {
            metadata.insert(key, value);
        }

        LogEntry {
            timestamp,
            level: record.level().to_string(),
            message,
            location,
            module_path,
            metadata,
        }
    }

    /// Format the current timestamp
    fn format_timestamp(&self) -> String {
        if let Some(format) = &self.timestamp_format {
            Local::now().format(format).to_string()
        } else {
            // Default to RFC3339
            Utc::now().to_rfc3339()
        }
    }

    /// Extract key-value pairs from the message
    fn extract_key_values(&self, message: &str) -> (String, Vec<(String, Value)>) {
        // Simple key-value extraction for now
        // Format: "message key1=value1 key2=value2"
        let parts: Vec<&str> = message.split(' ').collect();
        if parts.len() <= 1 {
            return (message.to_string(), Vec::new());
        }

        let mut kv_pairs = Vec::new();
        let mut message_parts = Vec::new();

        for part in parts {
            if part.contains('=') {
                let kv: Vec<&str> = part.splitn(2, '=').collect();
                if kv.len() == 2 {
                    let key = kv[0].to_string();
                    let value = kv[1].to_string();

                    // Try to parse as different types
                    if let Ok(num) = value.parse::<i64>() {
                        kv_pairs.push((key, Value::Number(num.into())));
                    } else if let Ok(num) = value.parse::<f64>() {
                        // Create a Number from f64, dealing with serde_json requirements
                        if let Some(num) = serde_json::Number::from_f64(num) {
                            kv_pairs.push((key, Value::Number(num)));
                        } else {
                            kv_pairs.push((key, Value::String(value)));
                        }
                    } else if value == "true" {
                        kv_pairs.push((key, Value::Bool(true)));
                    } else if value == "false" {
                        kv_pairs.push((key, Value::Bool(false)));
                    } else {
                        kv_pairs.push((key, Value::String(value)));
                    }
                    continue;
                }
            }

            message_parts.push(part);
        }

        (message_parts.join(" "), kv_pairs)
    }
}

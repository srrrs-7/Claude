# Structured Logger

A flexible, structured logging library for Rust applications that supports JSON, pretty, and compact output formats.

## Features

- Multiple output formats (JSON, Pretty, Compact)
- Structured logging with key-value pairs
- Global metadata for all log entries
- Context-based logging for request/transaction tracing
- File and stdout output support
- Timestamp customization
- Compatible with the standard `log` crate

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
structured_logger = "0.1.0"
log = "0.4"
```

## Quick Start

```rust
use structured_logger::init;
use log::{info, error};

fn main() {
    // Initialize with default settings
    init().expect("Failed to initialize logger");
    
    // Basic logging
    info!("Application started");
    
    // Logging with key-value pairs
    info!("User logged in user_id=123 role=admin");
    error!("Database connection failed host=db.example.com port=5432");
}
```

## Configuration

```rust
use structured_logger::{init_with_config, LoggerConfig, OutputFormat};
use log::{info, LevelFilter};

fn main() {
    let config = LoggerConfig::new()
        .with_level(LevelFilter::Debug)
        .with_format(OutputFormat::Json)
        .with_metadata("app_name", "my_app")
        .with_metadata("version", "1.0.0")
        .with_timestamp_format("%Y-%m-%d %H:%M:%S")
        .with_output_path("application.log");
    
    init_with_config(config).expect("Failed to initialize logger");
    
    info!("Configured logging system");
}
```

## Context-Based Logging

```rust
use structured_logger::{init, logger::LogContext};
use log::info;

fn main() {
    init().expect("Failed to initialize logger");
    
    let request_context = LogContext::new()
        .with_str("request_id", "req-123-abc")
        .with_str("user_id", "user-456")
        .with_str("ip", "192.168.1.1");
    
    request_context.info("Request received");
    
    // More logging...
}
```

## Output Formats

### JSON Format

```json
{"timestamp":"2023-01-01T12:34:56.789Z","level":"INFO","message":"User logged in","app_name":"example","version":"1.0.0","user_id":"123","role":"admin"}
```

### Pretty Format

Colorized, human-readable output for terminals:

```
2023-01-01T12:34:56.789Z [INFO] User logged in main.rs:15 [app_name=example, version=1.0.0, user_id=123, role=admin]
```

### Compact Format

```
2023-01-01T12:34:56.789Z [INFO] User logged in app_name=example,version=1.0.0,user_id=123,role=admin
```

## Advanced Usage

### Async Logging

For async applications using Tokio:

```rust
use structured_logger::{init_with_config, LoggerConfig};
use log::info;

#[tokio::main]
async fn main() {
    // Initialize logger
    init_with_config(LoggerConfig::default()).expect("Failed to initialize logger");
    
    // Spawn some tasks
    let task1 = tokio::spawn(async {
        info!("Task 1 running task_id=1");
        // Do work...
    });
    
    let task2 = tokio::spawn(async {
        info!("Task 2 running task_id=2");
        // Do work...
    });
    
    // Wait for tasks to complete
    let _ = tokio::join!(task1, task2);
    
    info!("All tasks completed");
}
```

### Extracting Key-Value Pairs

The logger automatically extracts key-value pairs from messages using a simple format:

```rust
info!("Database query executed query_time_ms=42 rows=1000");
```

This will produce a structured log with `query_time_ms` and `rows` as separate fields.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

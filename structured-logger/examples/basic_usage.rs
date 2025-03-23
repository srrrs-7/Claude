use structured_logger::{init_with_config, LoggerConfig, OutputFormat};
use log::{debug, error, info, warn, LevelFilter};

fn main() {
    // Initialize with JSON format
    let config = LoggerConfig::new()
        .with_level(LevelFilter::Debug)
        .with_format(OutputFormat::Json)
        .with_metadata("app_name", "example")
        .with_metadata("version", "1.0.0");
    
    init_with_config(config).expect("Failed to initialize logger");
    
    // Basic logging
    info!("Application started");
    debug!("Debug information");
    warn!("This is a warning");
    error!("An error occurred");
    
    // Logging with key-value pairs
    info!("User logged in user_id=123 role=admin");
    debug!("Database query executed query_time_ms=42 rows=1000");
    warn!("Slow request detected endpoint=/api/users duration_ms=1500 status=200");
    error!("Database connection failed host=db.example.com port=5432 retry_count=3");
    
    info!("Application stopped");
}

use structured_logger::{init, logger::LogContext};
use log::{info, LevelFilter};

fn main() {
    // Initialize with default settings
    init().expect("Failed to initialize logger");
    
    // Standard logging
    info!("Application started");
    
    // Create a context for a request
    let request_context = LogContext::new()
        .with_str("request_id", "req-123-abc")
        .with_str("user_agent", "Mozilla/5.0")
        .with_str("ip", "192.168.1.1");
    
    // Log with context
    request_context.info("Request received");
    
    // Process request...
    
    // Add more context for database operation
    let db_context = LogContext::new()
        .with_str("request_id", "req-123-abc")
        .with_str("db_operation", "query")
        .with_str("table", "users")
        .with_number("duration_ms", 42.5);
    
    db_context.info("Database operation completed");
    
    // For an error scenario
    let error_context = LogContext::new()
        .with_str("request_id", "req-123-abc")
        .with_str("operation", "payment")
        .with_number("amount", 99.99)
        .with_str("error_code", "INSUFFICIENT_FUNDS");
    
    error_context.error("Payment processing failed");
    
    info!("Application stopped");
}

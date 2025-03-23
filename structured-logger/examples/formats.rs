use structured_logger::{init_with_config, LoggerConfig, OutputFormat};
use log::{info, error, LevelFilter};

fn main() {
    // Try different output formats
    demonstrate_json_format();
    demonstrate_pretty_format();
    demonstrate_compact_format();
}

fn demonstrate_json_format() {
    let config = LoggerConfig::new()
        .with_level(LevelFilter::Debug)
        .with_format(OutputFormat::Json)
        .with_metadata("format", "json");
    
    init_with_config(config).expect("Failed to initialize logger");
    
    println!("\n=== JSON Format ===");
    info!("This is JSON formatted log message user_id=123");
    error!("Something went wrong error_code=500");
}

fn demonstrate_pretty_format() {
    let config = LoggerConfig::new()
        .with_level(LevelFilter::Debug)
        .with_format(OutputFormat::Pretty)
        .with_metadata("format", "pretty");
    
    init_with_config(config).expect("Failed to initialize logger");
    
    println!("\n=== Pretty Format ===");
    info!("This is a pretty formatted log message user_id=123");
    error!("Something went wrong error_code=500");
}

fn demonstrate_compact_format() {
    let config = LoggerConfig::new()
        .with_level(LevelFilter::Debug)
        .with_format(OutputFormat::Compact)
        .with_metadata("format", "compact");
    
    init_with_config(config).expect("Failed to initialize logger");
    
    println!("\n=== Compact Format ===");
    info!("This is a compact formatted log message user_id=123");
    error!("Something went wrong error_code=500");
}

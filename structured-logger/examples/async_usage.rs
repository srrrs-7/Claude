use structured_logger::{init_with_config, LoggerConfig, OutputFormat};
use log::{info, error, debug, LevelFilter};
use std::time::Duration;

#[tokio::main]
async fn main() {
    // Initialize with JSON format
    let config = LoggerConfig::new()
        .with_level(LevelFilter::Debug)
        .with_format(OutputFormat::Json)
        .with_metadata("app_name", "async_example")
        .with_metadata("version", "1.0.0");
    
    init_with_config(config).expect("Failed to initialize logger");
    
    info!("Async application started");
    
    // Spawn multiple tasks
    let task1 = tokio::spawn(async {
        info!("Task 1 started task_id=1");
        
        for i in 1..=3 {
            debug!("Task 1 progress iteration={} task_id=1", i);
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        
        info!("Task 1 completed task_id=1 status=success");
    });
    
    let task2 = tokio::spawn(async {
        info!("Task 2 started task_id=2");
        
        for i in 1..=3 {
            debug!("Task 2 progress iteration={} task_id=2", i);
            tokio::time::sleep(Duration::from_millis(150)).await;
        }
        
        error!("Task 2 failed task_id=2 error_code=500 error_message=timeout");
    });
    
    // Wait for tasks to complete
    let (result1, result2) = tokio::join!(task1, task2);
    
    match (result1, result2) {
        (Ok(_), Ok(_)) => info!("All tasks completed"),
        _ => error!("Some tasks failed"),
    }
    
    info!("Async application stopped");
}

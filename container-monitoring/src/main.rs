use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use tokio::signal;
use tokio::sync::Mutex;
use tokio::time;
use tracing::{info, warn};

// モジュールのインポート（mod.rsを使わない構造）
mod config;
mod docker;
mod metrics;
mod telemetry;
mod server;

use crate::config::{Config, load_config};
use crate::docker::DockerClient;
use crate::metrics::MetricsCollector;
use crate::telemetry::init_telemetry;
use crate::server::start_metrics_server;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, default_value = "config/config.toml")]
    config: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command-line arguments
    let args = Args::parse();
    
    // Load configuration
    let config = load_config(&args.config)?;
    
    // Initialize OpenTelemetry and logging
    let _telemetry_guard = init_telemetry(&config)?;
    
    // Log startup information
    info!(
        "Starting container-monitoring service (version: {})",
        env!("CARGO_PKG_VERSION")
    );
    info!("Interval set to {} seconds", config.general.interval);
    
    // Create docker client
    let docker_client = DockerClient::new(&config.docker)?;
    info!("Connected to Docker daemon");
    
    // Create metrics collector
    let metrics_collector = Arc::new(Mutex::new(
        MetricsCollector::new(docker_client, &config.metrics)?
    ));
    
    // Start metrics server for Prometheus scraping
    let metrics_collector_clone = metrics_collector.clone();
    let prometheus_handle = tokio::spawn(async move {
        if let Err(e) = start_metrics_server(metrics_collector_clone, config.telemetry.prometheus_port).await {
            warn!("Metrics server error: {}", e);
        }
    });
    
    // Start metrics collection loop
    let collection_interval = Duration::from_secs(config.general.interval);
    let collector_handle = tokio::spawn(async move {
        let mut interval = time::interval(collection_interval);
        
        loop {
            interval.tick().await;
            
            // Collect metrics
            info!("Collecting container metrics...");
            if let Err(e) = metrics_collector.lock().await.collect_metrics().await {
                warn!("Error collecting metrics: {}", e);
            }
        }
    });
    
    // Wait for shutdown signal
    match signal::ctrl_c().await {
        Ok(()) => {
            info!("Shutdown signal received, stopping service");
        }
        Err(err) => {
            warn!("Error listening for shutdown signal: {}", err);
        }
    }
    
    // Clean shutdown
    prometheus_handle.abort();
    collector_handle.abort();
    
    info!("Container monitoring service stopped");
    Ok(())
}
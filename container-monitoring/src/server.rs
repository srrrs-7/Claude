use anyhow::Result;
use prometheus::{Encoder, Registry, TextEncoder};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info, instrument, warn};
use warp::Filter;

use crate::metrics::MetricsCollector;

#[instrument(skip(metrics_collector), level = "info")]
pub async fn start_metrics_server(
    metrics_collector: Arc<Mutex<MetricsCollector>>,
    port: u16,
) -> Result<()> {
    info!("Starting metrics server on port {}", port);
    
    // Define routes
    let metrics_route = warp::path!("metrics")
        .and(warp::get())
        .map(move || {
            let registry = Registry::new();
            
            // The OpenTelemetry metrics are automatically exported via Prometheus
            // Just return the metrics endpoint content
            let mut buffer = Vec::new();
            let encoder = TextEncoder::new();
            
            // This collects from default Prometheus registry which OpenTelemetry integrates with
            let metric_families = prometheus::gather();
            if let Err(e) = encoder.encode(&metric_families, &mut buffer) {
                warn!("Could not encode metrics: {}", e);
            }
            
            String::from_utf8(buffer).unwrap_or_else(|_| "# Error encoding metrics".to_string())
        });
    
    let health_route = warp::path!("health")
        .and(warp::get())
        .map(|| "ok");
    
    let routes = metrics_route
        .or(health_route)
        .with(warp::log("metrics_server"));
    
    // Start the server
    warp::serve(routes)
        .run(([0, 0, 0, 0], port))
        .await;
    
    Ok(())
}
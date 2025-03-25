use anyhow::{Context, Result};
use opentelemetry::sdk::export::metrics::aggregation;
use opentelemetry::sdk::metrics::reader;
use opentelemetry::sdk::metrics::PeriodicReader;
use opentelemetry::sdk::{trace, Resource};
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use std::time::Duration;
use tracing::info;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

use crate::config::Config;

#[tracing::instrument(level = "info")]
pub fn init_telemetry(config: &Config) -> Result<impl Drop> {
    // Set up OpenTelemetry resource
    let resource = Resource::new(vec![
        KeyValue::new("service.name", config.telemetry.service_name.clone()),
        KeyValue::new("service.version", env!("CARGO_PKG_VERSION").to_string()),
    ]);

    // Configure OTLP exporter
    let otlp_exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(&config.telemetry.otel_endpoint);

    // Initialize OpenTelemetry tracing
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(otlp_exporter.clone())
        .with_trace_config(
            trace::config()
                .with_resource(resource.clone())
                .with_sampler(trace::Sampler::AlwaysOn),
        )
        .install_batch(opentelemetry::runtime::Tokio)
        .context("Failed to initialize OpenTelemetry tracer")?;

    // Initialize OpenTelemetry metrics
    let meter_provider = opentelemetry_otlp::new_pipeline()
        .metrics(opentelemetry::runtime::Tokio)
        .with_exporter(otlp_exporter)
        .with_resource(resource)
        .with_period(Duration::from_secs(10))
        .build()
        .context("Failed to initialize OpenTelemetry metrics")?;

    // Set as global meter provider
    let _meter_provider_guard = opentelemetry::global::set_meter_provider(meter_provider);

    // Set up tracing subscriber
    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(&config.logging.level))
        .unwrap_or_else(|_| EnvFilter::new("info"));

    // Create a tracing layer with the configured tracer
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    // Create the subscriber and set it as global default
    tracing_subscriber::registry()
        .with(env_filter)
        .with(telemetry)
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!(
        "Telemetry initialized with OTLP exporter at {}",
        config.telemetry.otel_endpoint
    );

    // Return a guard that will flush telemetry on drop
    Ok(TelemetryGuard {})
}

// Helper struct that will flush telemetry on drop
pub struct TelemetryGuard {}

impl Drop for TelemetryGuard {
    fn drop(&mut self) {
        info!("Shutting down telemetry");
        opentelemetry::global::shutdown_tracer_provider();
        opentelemetry::global::shutdown_meter_provider();
    }
}

use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub general: GeneralConfig,
    pub telemetry: TelemetryConfig,
    pub docker: DockerConfig,
    pub metrics: MetricsConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GeneralConfig {
    pub interval: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TelemetryConfig {
    pub service_name: String,
    pub otel_exporter: String,
    pub otel_endpoint: String,
    pub prometheus_port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DockerConfig {
    pub socket_path: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MetricsConfig {
    pub enable_cpu: bool,
    pub enable_memory: bool,
    pub enable_network: bool,
    pub enable_disk: bool,
    #[serde(default)]
    pub container_filters: ContainerFilters,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ContainerFilters {
    /// 特定のコンテナIDリスト。指定されていれば、これらのコンテナのみを監視します
    #[serde(default)]
    pub container_ids: Vec<String>,
    
    /// 特定のコンテナ名パターン。指定されていれば、一致するコンテナのみを監視します
    #[serde(default)]
    pub name_patterns: Vec<String>,
    
    /// 特定のイメージパターン。指定されていれば、一致するイメージのコンテナのみを監視します
    #[serde(default)]
    pub image_patterns: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoggingConfig {
    pub level: String,
}

pub fn load_config<P: AsRef<Path>>(path: P) -> Result<Config> {
    let config_str = fs::read_to_string(path.as_ref())
        .with_context(|| format!("Failed to read config file: {:?}", path.as_ref()))?;
    
    let config: Config = toml::from_str(&config_str)
        .with_context(|| "Failed to parse config file")?;
    
    Ok(config)
}
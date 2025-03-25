use anyhow::Result;
use opentelemetry::metrics::{Counter, Histogram, MeterProvider, Unit, UpDownCounter};
use opentelemetry_sdk::metrics::{Aggregation, Instrument, Stream};
use opentelemetry_sdk::metrics::reader::DefaultAggregationSelector;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info, instrument, warn};

use opentelemetry::metrics::Meter;
use opentelemetry::KeyValue;

use crate::config::MetricsConfig;
use crate::docker::{ContainerInfo, DockerClient};

pub struct MetricsCollector {
    docker_client: DockerClient,
    config: MetricsConfig,
    meter: Meter,
    
    // Metrics
    cpu_usage: Histogram<f64>,
    memory_usage: UpDownCounter<u64>,
    memory_limit: UpDownCounter<u64>,
    memory_usage_percent: Histogram<f64>,
    network_receive_bytes: Counter<u64>,
    network_transmit_bytes: Counter<u64>,
    fs_reads_bytes: Counter<u64>,
    fs_writes_bytes: Counter<u64>,
    container_count: UpDownCounter<i64>,
    
    // Track previous values for counters to calculate deltas
    prev_network_rx: Arc<Mutex<HashMap<String, u64>>>,
    prev_network_tx: Arc<Mutex<HashMap<String, u64>>>,
    prev_fs_reads: Arc<Mutex<HashMap<String, u64>>>,
    prev_fs_writes: Arc<Mutex<HashMap<String, u64>>>,
}

impl MetricsCollector {
    pub fn new(docker_client: DockerClient, config: &MetricsConfig) -> Result<Self> {
        let meter = opentelemetry::global::meter("container-monitoring");
        
        // Create metrics instruments
        let cpu_usage = meter
            .f64_histogram("container_cpu_usage_percent")
            .with_description("CPU usage in percent")
            .with_unit(Unit::new("%"))
            .init();
            
        let memory_usage = meter
            .u64_up_down_counter("container_memory_usage_bytes")
            .with_description("Memory usage in bytes")
            .with_unit(Unit::new("By"))
            .init();
            
        let memory_limit = meter
            .u64_up_down_counter("container_memory_limit_bytes")
            .with_description("Memory limit in bytes")
            .with_unit(Unit::new("By"))
            .init();
            
        let memory_usage_percent = meter
            .f64_histogram("container_memory_usage_percent")
            .with_description("Memory usage in percent")
            .with_unit(Unit::new("%"))
            .init();
            
        let network_receive_bytes = meter
            .u64_counter("container_network_receive_bytes_total")
            .with_description("Network bytes received")
            .with_unit(Unit::new("By"))
            .init();
            
        let network_transmit_bytes = meter
            .u64_counter("container_network_transmit_bytes_total")
            .with_description("Network bytes transmitted")
            .with_unit(Unit::new("By"))
            .init();
            
        let fs_reads_bytes = meter
            .u64_counter("container_fs_reads_bytes_total")
            .with_description("Filesystem bytes read")
            .with_unit(Unit::new("By"))
            .init();
            
        let fs_writes_bytes = meter
            .u64_counter("container_fs_writes_bytes_total")
            .with_description("Filesystem bytes written")
            .with_unit(Unit::new("By"))
            .init();
            
        let container_count = meter
            .i64_up_down_counter("container_count")
            .with_description("Number of containers")
            .init();
        
        Ok(Self {
            docker_client,
            config: config.clone(),
            meter,
            cpu_usage,
            memory_usage,
            memory_limit,
            memory_usage_percent,
            network_receive_bytes,
            network_transmit_bytes,
            fs_reads_bytes,
            fs_writes_bytes,
            container_count,
            prev_network_rx: Arc::new(Mutex::new(HashMap::new())),
            prev_network_tx: Arc::new(Mutex::new(HashMap::new())),
            prev_fs_reads: Arc::new(Mutex::new(HashMap::new())),
            prev_fs_writes: Arc::new(Mutex::new(HashMap::new())),
        })
    }
    
    #[instrument(skip(self), level = "debug")]
    pub async fn collect_metrics(&mut self) -> Result<()> {
        // Get list of containers (filtered if necessary)
        let mut containers = self.docker_client.list_filtered_containers(&self.config.container_filters).await?;
        
        // Update container count metrics
        let running_containers = containers.iter().filter(|c| c.status == "running").count() as i64;
        let total_containers = containers.len() as i64;
        
        self.container_count.add(running_containers, &[KeyValue::new("status", "running")]);
        self.container_count.add(total_containers - running_containers, &[KeyValue::new("status", "not_running")]);
        
        // Collect stats for running containers
        self.docker_client.collect_container_stats(&mut containers).await?;
        
        // Process container stats and record metrics
        for container in containers.iter().filter(|c| c.status == "running") {
            if let Some(stats) = &container.stats {
                let labels = &[
                    KeyValue::new("container_id", container.id.clone()),
                    KeyValue::new("container_name", container.name.clone()),
                    KeyValue::new("image", container.image.clone()),
                ];
                
                // CPU metrics
                if self.config.enable_cpu {
                    self.cpu_usage.record(stats.cpu_usage_percent, labels);
                }
                
                // Memory metrics
                if self.config.enable_memory {
                    self.memory_usage.add(stats.memory_usage_bytes, labels);
                    self.memory_limit.add(stats.memory_limit_bytes, labels);
                    self.memory_usage_percent.record(stats.memory_usage_percent, labels);
                }
                
                // Network metrics
                if self.config.enable_network {
                    let mut prev_rx = self.prev_network_rx.lock().await;
                    let mut prev_tx = self.prev_network_tx.lock().await;
                    
                    let rx_delta = self.calculate_delta(&mut prev_rx, &container.id, stats.network_rx_bytes);
                    let tx_delta = self.calculate_delta(&mut prev_tx, &container.id, stats.network_tx_bytes);
                    
                    if rx_delta > 0 {
                        self.network_receive_bytes.add(rx_delta, labels);
                    }
                    
                    if tx_delta > 0 {
                        self.network_transmit_bytes.add(tx_delta, labels);
                    }
                }
                
                // Disk I/O metrics
                if self.config.enable_disk {
                    let mut prev_reads = self.prev_fs_reads.lock().await;
                    let mut prev_writes = self.prev_fs_writes.lock().await;
                    
                    let reads_delta = self.calculate_delta(&mut prev_reads, &container.id, stats.block_read_bytes);
                    let writes_delta = self.calculate_delta(&mut prev_writes, &container.id, stats.block_write_bytes);
                    
                    if reads_delta > 0 {
                        self.fs_reads_bytes.add(reads_delta, labels);
                    }
                    
                    if writes_delta > 0 {
                        self.fs_writes_bytes.add(writes_delta, labels);
                    }
                }
            }
        }
        
        // Clean up previous values for containers that no longer exist
        self.cleanup_previous_values(&containers).await;
        
        Ok(())
    }
    
    #[instrument(skip(self, prev_values), fields(container_id = container_id, current_value = current_value), level = "debug")]
    fn calculate_delta(&self, prev_values: &mut HashMap<String, u64>, container_id: &str, current_value: u64) -> u64 {
        let prev_value = prev_values.get(container_id).copied().unwrap_or(0);
        
        // Update previous value
        prev_values.insert(container_id.to_string(), current_value);
        
        // Calculate delta, handle counter reset
        if current_value >= prev_value {
            current_value - prev_value
        } else {
            current_value
        }
    }
    
    #[instrument(skip(self, containers), level = "debug")]
    async fn cleanup_previous_values(&self, containers: &[ContainerInfo]) {
        let container_ids: std::collections::HashSet<String> = containers
            .iter()
            .map(|c| c.id.clone())
            .collect();
        
        // Clean up each previous value map
        {
            let mut prev_rx = self.prev_network_rx.lock().await;
            prev_rx.retain(|id, _| container_ids.contains(id));
        }
        
        {
            let mut prev_tx = self.prev_network_tx.lock().await;
            prev_tx.retain(|id, _| container_ids.contains(id));
        }
        
        {
            let mut prev_reads = self.prev_fs_reads.lock().await;
            prev_reads.retain(|id, _| container_ids.contains(id));
        }
        
        {
            let mut prev_writes = self.prev_fs_writes.lock().await;
            prev_writes.retain(|id, _| container_ids.contains(id));
        }
    }
}
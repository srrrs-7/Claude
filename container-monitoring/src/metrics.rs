use anyhow::Result;
use opentelemetry::metrics::{Counter, Histogram, MeterProvider, Unit, UpDownCounter};
use opentelemetry::KeyValue;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info, instrument, warn};

use crate::config::MetricsConfig;
use crate::docker::{ContainerInfo, DockerClient};

/// メトリクスコレクター - Dockerメトリクスの収集とOpenTelemetryへの変換を担当
pub struct MetricsCollector {
    docker_client: DockerClient,
    config: MetricsConfig,
    
    // OpenTelemetryメーター
    meter: opentelemetry::metrics::Meter,
    
    // メトリクスインストゥルメント
    cpu_usage: Histogram<f64>,
    memory_usage: UpDownCounter<u64>,
    memory_limit: UpDownCounter<u64>,
    memory_usage_percent: Histogram<f64>,
    network_receive_bytes: Counter<u64>,
    network_transmit_bytes: Counter<u64>,
    fs_reads_bytes: Counter<u64>,
    fs_writes_bytes: Counter<u64>,
    container_count: UpDownCounter<i64>,
    
    // カウンター型メトリクスのための前回値（デルタ計算用）
    prev_network_rx: Arc<Mutex<HashMap<String, u64>>>,
    prev_network_tx: Arc<Mutex<HashMap<String, u64>>>,
    prev_fs_reads: Arc<Mutex<HashMap<String, u64>>>,
    prev_fs_writes: Arc<Mutex<HashMap<String, u64>>>,
}

impl MetricsCollector {
    /// 新しいメトリクスコレクターを作成
    pub fn new(docker_client: DockerClient, config: &MetricsConfig) -> Result<Self> {
        debug!("Initializing metrics collector");
        let meter = opentelemetry::global::meter("container-monitoring");
        
        // メトリクスインストゥルメントの初期化
        let cpu_usage = Self::init_cpu_metric(&meter);
        let (memory_usage, memory_limit, memory_usage_percent) = Self::init_memory_metrics(&meter);
        let (network_receive_bytes, network_transmit_bytes) = Self::init_network_metrics(&meter);
        let (fs_reads_bytes, fs_writes_bytes) = Self::init_fs_metrics(&meter);
        let container_count = Self::init_container_count_metric(&meter);
        
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
    
    // CPUメトリクスのインストゥルメントを初期化
    fn init_cpu_metric(meter: &opentelemetry::metrics::Meter) -> Histogram<f64> {
        meter
            .f64_histogram("container_cpu_usage_percent")
            .with_description("CPU usage in percent")
            .with_unit(Unit::new("%"))
            .init()
    }
    
    // メモリメトリクスのインストゥルメントを初期化
    fn init_memory_metrics(meter: &opentelemetry::metrics::Meter) -> (UpDownCounter<u64>, UpDownCounter<u64>, Histogram<f64>) {
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
            
        (memory_usage, memory_limit, memory_usage_percent)
    }
    
    // ネットワークメトリクスのインストゥルメントを初期化
    fn init_network_metrics(meter: &opentelemetry::metrics::Meter) -> (Counter<u64>, Counter<u64>) {
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
            
        (network_receive_bytes, network_transmit_bytes)
    }
    
    // ファイルシステムメトリクスのインストゥルメントを初期化
    fn init_fs_metrics(meter: &opentelemetry::metrics::Meter) -> (Counter<u64>, Counter<u64>) {
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
            
        (fs_reads_bytes, fs_writes_bytes)
    }
    
    // コンテナ数メトリクスのインストゥルメントを初期化
    fn init_container_count_metric(meter: &opentelemetry::metrics::Meter) -> UpDownCounter<i64> {
        meter
            .i64_up_down_counter("container_count")
            .with_description("Number of containers")
            .init()
    }
    
    /// メトリクスを収集
    #[instrument(skip(self), level = "debug")]
    pub async fn collect_metrics(&mut self) -> Result<()> {
        debug!("Starting metrics collection cycle");
        // フィルタに従ってコンテナのリストを取得
        let mut containers = self.docker_client.list_filtered_containers(&self.config.container_filters).await?;
        
        // コンテナ数メトリクスを更新
        self.update_container_count_metrics(&containers);
        
        // 実行中のコンテナの統計情報を収集
        self.docker_client.collect_container_stats(&mut containers).await?;
        
        // メトリクスを処理して記録
        self.process_metrics(&containers).await?;
        
        // 古い値を削除（存在しなくなったコンテナ）
        self.cleanup_previous_values(&containers).await;
        
        debug!("Metrics collection cycle completed");
        Ok(())
    }
    
    // コンテナ数メトリクスを更新
    fn update_container_count_metrics(&self, containers: &[ContainerInfo]) {
        let running_containers = containers.iter().filter(|c| c.status == "running").count() as i64;
        let total_containers = containers.len() as i64;
        
        self.container_count.add(running_containers, &[KeyValue::new("status", "running")]);
        self.container_count.add(total_containers - running_containers, &[KeyValue::new("status", "not_running")]);
        
        debug!(
            running = running_containers,
            total = total_containers,
            "Container count metrics updated"
        );
    }
    
    // 収集したメトリクスを処理して記録
    async fn process_metrics(&self, containers: &[ContainerInfo]) -> Result<()> {
        for container in containers.iter().filter(|c| c.status == "running") {
            if let Some(stats) = &container.stats {
                let labels = &[
                    KeyValue::new("container_id", container.id.clone()),
                    KeyValue::new("container_name", container.name.clone()),
                    KeyValue::new("image", container.image.clone()),
                ];
                
                // 有効化されたメトリクスを処理
                self.process_enabled_metrics(container, stats, labels).await?;
            }
        }
        
        Ok(())
    }
    
    // 有効化されたメトリクスのみを処理
    async fn process_enabled_metrics(&self, container: &ContainerInfo, stats: &crate::docker::ContainerStats, labels: &[KeyValue]) -> Result<()> {
        // CPU メトリクス
        if self.config.enable_cpu {
            self.cpu_usage.record(stats.cpu_usage_percent, labels);
        }
        
        // メモリ メトリクス
        if self.config.enable_memory {
            self.memory_usage.add(stats.memory_usage_bytes, labels);
            self.memory_limit.add(stats.memory_limit_bytes, labels);
            self.memory_usage_percent.record(stats.memory_usage_percent, labels);
        }
        
        // ネットワーク メトリクス
        if self.config.enable_network {
            self.process_network_metrics(container, stats, labels).await?;
        }
        
        // ディスク I/O メトリクス
        if self.config.enable_disk {
            self.process_disk_metrics(container, stats, labels).await?;
        }
        
        Ok(())
    }
    
    // ネットワークメトリクスを処理
    async fn process_network_metrics(&self, container: &ContainerInfo, stats: &crate::docker::ContainerStats, labels: &[KeyValue]) -> Result<()> {
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
        
        Ok(())
    }
    
    // ディスクメトリクスを処理
    async fn process_disk_metrics(&self, container: &ContainerInfo, stats: &crate::docker::ContainerStats, labels: &[KeyValue]) -> Result<()> {
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
        
        Ok(())
    }
    
    /// カウンター型メトリクスのためのデルタ値を計算
    #[instrument(skip(self, prev_values), fields(container_id = container_id, current_value = current_value), level = "debug")]
    fn calculate_delta(&self, prev_values: &mut HashMap<String, u64>, container_id: &str, current_value: u64) -> u64 {
        let prev_value = prev_values.get(container_id).copied().unwrap_or(0);
        
        // 前回値を更新
        prev_values.insert(container_id.to_string(), current_value);
        
        // デルタを計算（カウンターリセット対応）
        if current_value >= prev_value {
            current_value - prev_value
        } else {
            // カウンターリセットの場合は現在値をそのまま返す
            debug!("Counter reset detected for {}", container_id);
            current_value
        }
    }
    
    /// 存在しなくなったコンテナの前回値を削除
    #[instrument(skip(self, containers), level = "debug")]
    async fn cleanup_previous_values(&self, containers: &[ContainerInfo]) {
        let container_ids: HashSet<String> = containers
            .iter()
            .map(|c| c.id.clone())
            .collect();
        
        let mut cleaned_up = 0;
        
        // 各前回値マップをクリーンアップ
        {
            let mut prev_rx = self.prev_network_rx.lock().await;
            cleaned_up += Self::cleanup_previous_map(&mut prev_rx, &container_ids);
        }
        
        {
            let mut prev_tx = self.prev_network_tx.lock().await;
            cleaned_up += Self::cleanup_previous_map(&mut prev_tx, &container_ids);
        }
        
        {
            let mut prev_reads = self.prev_fs_reads.lock().await;
            cleaned_up += Self::cleanup_previous_map(&mut prev_reads, &container_ids);
        }
        
        {
            let mut prev_writes = self.prev_fs_writes.lock().await;
            cleaned_up += Self::cleanup_previous_map(&mut prev_writes, &container_ids);
        }
        
        if cleaned_up > 0 {
            debug!("Cleaned up {} previous value entries", cleaned_up);
        }
    }
    
    // 前回値マップのクリーンアップヘルパー
    fn cleanup_previous_map(map: &mut HashMap<String, u64>, valid_ids: &HashSet<String>) -> usize {
        let before_count = map.len();
        map.retain(|id, _| valid_ids.contains(id));
        before_count - map.len()
    }
}
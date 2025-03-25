use anyhow::{Context, Result, anyhow};
use bollard::container::{ListContainersOptions, Stats, StatsOptions};
use bollard::Docker;
use bollard::system::SystemInfo;
use futures::stream::StreamExt;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, instrument, Instrument as TracingInstrument, Span};

use crate::config::{DockerConfig, ContainerFilters};

pub struct DockerClient {
    client: Docker,
}

#[derive(Debug, Clone)]
pub struct ContainerInfo {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: String,
    pub stats: Option<ContainerStats>,
}

#[derive(Debug, Clone, Default)]
pub struct ContainerStats {
    pub cpu_usage_percent: f64,
    pub memory_usage_bytes: u64,
    pub memory_limit_bytes: u64,
    pub memory_usage_percent: f64,
    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
    pub block_read_bytes: u64,
    pub block_write_bytes: u64,
    pub pids: u64,
}

impl DockerClient {
    pub fn new(config: &DockerConfig) -> Result<Self> {
        let client = Docker::connect_with_socket(&config.socket_path, 120, bollard::API_DEFAULT_VERSION)
            .with_context(|| format!("Failed to connect to Docker socket at {}", config.socket_path))?;
        
        Ok(Self { client })
    }
    
    #[instrument(skip(self), level = "debug")]
    pub async fn get_info(&self) -> Result<SystemInfo> {
        debug!("Retrieving Docker system info");
        let result = self.client.info()
            .await
            .with_context(|| "Failed to get Docker system info");
            
        if let Ok(ref info) = result {
            debug!(containers = info.containers, 
                   images = info.images, 
                   driver = ?info.driver, 
                   "Docker system info retrieved");
        }
        
        result
    }
    
    #[instrument(skip(self), level = "debug")]
    pub async fn list_containers(&self) -> Result<Vec<ContainerInfo>> {
        let options = Some(ListContainersOptions::<String>{
            all: true,
            ..Default::default()
        });
        
        let containers = self.client.list_containers(options)
            .await
            .with_context(|| "Failed to list containers")?;
        
        let mut container_infos = Vec::new();
        
        for container in containers {
            let id = container.id.unwrap_or_default();
            let name = container.names
                .unwrap_or_default()
                .first()
                .cloned()
                .unwrap_or_default()
                .trim_start_matches('/')
                .to_string();
            
            let image = container.image.unwrap_or_default();
            let status = container.state.unwrap_or_default();
            
            container_infos.push(ContainerInfo {
                id,
                name,
                image,
                status,
                stats: None,
            });
        }
        
        Ok(container_infos)
    }
    
    /// 指定されたフィルターに基づいてコンテナを取得します
    #[instrument(skip(self), level = "debug")]
    pub async fn list_filtered_containers(&self, filters: &ContainerFilters) -> Result<Vec<ContainerInfo>> {
        // すべてのコンテナを取得
        let all_containers = self.list_containers().await?;
        
        // フィルターが空の場合はすべてのコンテナを返す
        if filters.container_ids.is_empty() && filters.name_patterns.is_empty() && filters.image_patterns.is_empty() {
            return Ok(all_containers);
        }
        
        // フィルターに従ってコンテナをフィルタリング
        let filtered_containers = all_containers.into_iter()
            .filter(|container| {
                // IDでフィルタリング
                if !filters.container_ids.is_empty() && filters.container_ids.contains(&container.id) {
                    return true;
                }
                
                // 名前パターンでフィルタリング
                for pattern in &filters.name_patterns {
                    if Self::matches_glob_pattern(&container.name, pattern) {
                        return true;
                    }
                }
                
                // イメージパターンでフィルタリング
                for pattern in &filters.image_patterns {
                    if Self::matches_glob_pattern(&container.image, pattern) {
                        return true;
                    }
                }
                
                // すべてのフィルターに一致しなかった場合はfalse
                false
            })
            .collect();
        
        Ok(filtered_containers)
    }
    
    // シンプルなグロブパターンマッチング
    fn matches_glob_pattern(input: &str, pattern: &str) -> bool {
        // シンプルなワイルドカードマッチング（*と?のみサポート）
        let pattern_chars: Vec<char> = pattern.chars().collect();
        let input_chars: Vec<char> = input.chars().collect();
        
        let mut dp = vec![vec![false; input_chars.len() + 1]; pattern_chars.len() + 1];
        dp[0][0] = true;  // 空パターンは空入力にマッチする
        
        // 先頭の*は空文字列にマッチする可能性がある
        for i in 1..=pattern_chars.len() {
            if pattern_chars[i-1] == '*' {
                dp[i][0] = dp[i-1][0];
            }
        }
        
        for i in 1..=pattern_chars.len() {
            for j in 1..=input_chars.len() {
                match pattern_chars[i-1] {
                    '*' => {
                        // *は0文字以上の任意の文字列にマッチ
                        dp[i][j] = dp[i-1][j] || dp[i][j-1];
                    },
                    '?' => {
                        // ?は任意の1文字にマッチ
                        dp[i][j] = dp[i-1][j-1];
                    },
                    pc => {
                        // 通常の文字は完全一致
                        dp[i][j] = dp[i-1][j-1] && pc == input_chars[j-1];
                    }
                }
            }
        }
        
        dp[pattern_chars.len()][input_chars.len()]
    }
    
    #[instrument(skip(self, containers), fields(container_count = containers.len()), level = "debug")]
    pub async fn collect_container_stats(&self, containers: &mut [ContainerInfo]) -> Result<()> {
        // Collect stats for each container concurrently using a vector
        let mut stats_futures = Vec::new();
        
        for container in containers.iter() {
            // Only collect stats for running containers
            if container.status != "running" {
                continue;
            }
            
            let container_id = container.id.clone();
            let client = self.client.clone();
            
            let future = async move {
                let stats_result = Self::get_container_stats(&client, &container_id).await;
                (container_id, stats_result)
            };
            
            stats_futures.push(future);
        }
        
        // Wait for all stats collection to complete
        let results: Vec<(String, Result<ContainerStats>)> = futures::future::join_all(stats_futures).await;
        
        // Create a map of container ID to stats
        let stats_map: HashMap<String, ContainerStats> = results
            .into_iter()
            .filter_map(|(id, result)| {
                match result {
                    Ok(stats) => Some((id, stats)),
                    Err(e) => {
                        error!("Failed to collect stats for container {}: {}", id, e);
                        None
                    }
                }
            })
            .collect();
        
        // Update container stats
        for container in containers.iter_mut() {
            if let Some(stats) = stats_map.get(&container.id) {
                container.stats = Some(stats.clone());
            }
        }
        
        Ok(())
    }
    
    #[instrument(skip(client), fields(container_id = container_id), level = "debug")]
    async fn get_container_stats(client: &Docker, container_id: &str) -> Result<ContainerStats> {
        let mut stats_stream = client.stats(
            container_id,
            Some(StatsOptions {
                stream: false,
                ..Default::default()
            }),
        );
        
        if let Some(stats_result) = stats_stream.next().await {
            match stats_result {
                Ok(stats) => {
                    return Ok(Self::parse_container_stats(stats));
                }
                Err(e) => {
                    return Err(anyhow!("Failed to get stats: {}", e));
                }
            }
        }
        
        Err(anyhow!("No stats received"))
    }
    
    fn parse_container_stats(stats: Stats) -> ContainerStats {
        let cpu_delta = stats.cpu_stats.cpu_usage.total_usage as f64 - 
                        stats.precpu_stats.cpu_usage.total_usage as f64;
        
        let system_cpu_delta = stats.cpu_stats.system_cpu_usage.unwrap_or(0) as f64 - 
                               stats.precpu_stats.system_cpu_usage.unwrap_or(0) as f64;
        
        let num_cpus = stats.cpu_stats.online_cpus.unwrap_or(1) as f64;
        
        let cpu_usage_percent = if system_cpu_delta > 0.0 && cpu_delta > 0.0 {
            (cpu_delta / system_cpu_delta) * num_cpus * 100.0
        } else {
            0.0
        };
        
        let memory_usage_bytes = stats.memory_stats.usage.unwrap_or(0);
        let memory_limit_bytes = stats.memory_stats.limit.unwrap_or(0);
        
        let memory_usage_percent = if memory_limit_bytes > 0 {
            (memory_usage_bytes as f64 / memory_limit_bytes as f64) * 100.0
        } else {
            0.0
        };
        
        // Network stats
        let mut network_rx_bytes = 0;
        let mut network_tx_bytes = 0;
        
        if let Some(networks) = stats.networks {
            for (_, network) = networks {
                network_rx_bytes += network.rx_bytes;
                network_tx_bytes += network.tx_bytes;
            }
        }
        
        // Block I/O stats
        let mut block_read_bytes = 0;
        let mut block_write_bytes = 0;
        
        if let Some(io_stats) = stats.blkio_stats.io_service_bytes_recursive {
            for io_stat in io_stats {
                match io_stat.op.as_deref() {
                    Some("Read") => block_read_bytes += io_stat.value.unwrap_or(0),
                    Some("Write") => block_write_bytes += io_stat.value.unwrap_or(0),
                    _ => {}
                }
            }
        }
        
        ContainerStats {
            cpu_usage_percent,
            memory_usage_bytes,
            memory_limit_bytes,
            memory_usage_percent,
            network_rx_bytes,
            network_tx_bytes,
            block_read_bytes,
            block_write_bytes,
            pids: stats.pids_stats.current.unwrap_or(0),
        }
    }
}
use anyhow::{Context, Result};
use bollard::container::{ListContainersOptions, Stats, StatsOptions};
use bollard::Docker;
use bollard::system::SystemInfo;
use futures::stream::StreamExt;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, instrument};

use crate::config::{DockerConfig, ContainerFilters};

/// Dockerクライアント - Docker APIとの通信を担当
pub struct DockerClient {
    client: Docker,
}

/// コンテナ情報の構造体
#[derive(Debug, Clone)]
pub struct ContainerInfo {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: String,
    pub stats: Option<ContainerStats>,
}

/// コンテナの統計情報の構造体
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
    /// 新しいDockerクライアントを作成
    pub fn new(config: &DockerConfig) -> Result<Self> {
        let client = Docker::connect_with_socket(&config.socket_path, 120, bollard::API_DEFAULT_VERSION)
            .with_context(|| format!("Failed to connect to Docker socket at {}", config.socket_path))?;
        
        Ok(Self { client })
    }
    
    /// Dockerシステム情報を取得
    #[instrument(skip(self), level = "debug")]
    pub async fn get_info(&self) -> Result<SystemInfo> {
        debug!("Retrieving Docker system info");
        let result = self.client.info()
            .await
            .with_context(|| "Failed to get Docker system info");
            
        if let Ok(ref info) = result {
            debug!(
                containers = info.containers, 
                images = info.images, 
                driver = ?info.driver, 
                "Docker system info retrieved"
            );
        }
        
        result
    }
    
    /// すべてのコンテナのリストを取得
    #[instrument(skip(self), level = "debug")]
    pub async fn list_containers(&self) -> Result<Vec<ContainerInfo>> {
        debug!("Listing all containers");
        let options = Some(ListContainersOptions::<String>{
            all: true,
            ..Default::default()
        });
        
        let containers = self.client.list_containers(options)
            .await
            .with_context(|| "Failed to list containers")?;
        
        let container_infos = containers.into_iter()
            .map(|container| {
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
                
                ContainerInfo {
                    id,
                    name,
                    image,
                    status,
                    stats: None,
                }
            })
            .collect();
        
        debug!(container_count = container_infos.len(), "Containers listed");
        Ok(container_infos)
    }
    
    /// フィルタに従ってコンテナのリストを取得
    #[instrument(skip(self), level = "debug")]
    pub async fn list_filtered_containers(&self, filters: &ContainerFilters) -> Result<Vec<ContainerInfo>> {
        // すべてのコンテナを取得
        let all_containers = self.list_containers().await?;
        
        // フィルタが空の場合はすべてのコンテナを返す
        if filters.container_ids.is_empty() && filters.name_patterns.is_empty() && filters.image_patterns.is_empty() {
            debug!("No filters applied, returning all containers");
            return Ok(all_containers);
        }
        
        // フィルタに従ってコンテナをフィルタリング
        let filtered_containers = self.apply_filters(all_containers, filters);
        debug!(
            original_count = all_containers.len(),
            filtered_count = filtered_containers.len(),
            "Containers filtered"
        );
        
        Ok(filtered_containers)
    }

    /// フィルタを適用してコンテナをフィルタリング
    fn apply_filters(&self, containers: Vec<ContainerInfo>, filters: &ContainerFilters) -> Vec<ContainerInfo> {
        containers.into_iter()
            .filter(|container| {
                // IDでフィルタリング
                if !filters.container_ids.is_empty() && filters.container_ids.contains(&container.id) {
                    return true;
                }
                
                // 名前パターンでフィルタリング
                for pattern in &filters.name_patterns {
                    if Self::matches_pattern(&container.name, pattern) {
                        return true;
                    }
                }
                
                // イメージパターンでフィルタリング
                for pattern in &filters.image_patterns {
                    if Self::matches_pattern(&container.image, pattern) {
                        return true;
                    }
                }
                
                // すべてのフィルタに一致しなかった場合はfalse
                false
            })
            .collect()
    }
    
    /// シンプルなパターンマッチング
    fn matches_pattern(input: &str, pattern: &str) -> bool {
        // '*'と'?'のみをサポートするシンプルなワイルドカードマッチング
        if pattern.contains('*') || pattern.contains('?') {
            Self::matches_wildcard(input, pattern)
        } else {
            // 完全一致
            input == pattern
        }
    }

    /// ワイルドカードパターンマッチング
    fn matches_wildcard(input: &str, pattern: &str) -> bool {
        let pattern_chars: Vec<char> = pattern.chars().collect();
        let input_chars: Vec<char> = input.chars().collect();
        
        // 動的計画法を使用したワイルドカードマッチング
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
    
    /// コンテナの統計情報を収集
    #[instrument(skip(self, containers), fields(container_count = containers.len()), level = "debug")]
    pub async fn collect_container_stats(&self, containers: &mut [ContainerInfo]) -> Result<()> {
        debug!("Collecting stats for {} containers", containers.len());
        let running_containers = containers.iter()
            .filter(|c| c.status == "running")
            .count();
        debug!("Found {} running containers", running_containers);
        
        // Collect stats for each container concurrently using a vector
        let stats_futures = containers.iter()
            .filter(|c| c.status == "running")
            .map(|container| {
                let container_id = container.id.clone();
                let client = self.client.clone();
                
                async move {
                    let stats_result = Self::get_container_stats(&client, &container_id).await;
                    (container_id, stats_result)
                }
            })
            .collect::<Vec<_>>();
        
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
        
        debug!("Successfully collected stats for {} containers", stats_map.len());
        Ok(())
    }
    
    /// 単一コンテナの統計情報を取得
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
                    let container_stats = Self::parse_container_stats(stats);
                    debug!(
                        cpu = container_stats.cpu_usage_percent,
                        memory_mb = container_stats.memory_usage_bytes / (1024 * 1024),
                        "Container stats retrieved"
                    );
                    Ok(container_stats)
                }
                Err(e) => {
                    error!("Failed to get stats: {}", e);
                    Err(anyhow::anyhow!("Failed to get stats: {}", e))
                }
            }
        } else {
            error!("No stats received for container {}", container_id);
            Err(anyhow::anyhow!("No stats received"))
        }
    }
    
    /// Docker APIから取得した統計情報を解析
    fn parse_container_stats(stats: Stats) -> ContainerStats {
        // CPU使用率の計算
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
        
        // メモリ使用率の計算
        let memory_usage_bytes = stats.memory_stats.usage.unwrap_or(0);
        let memory_limit_bytes = stats.memory_stats.limit.unwrap_or(0);
        
        let memory_usage_percent = if memory_limit_bytes > 0 {
            (memory_usage_bytes as f64 / memory_limit_bytes as f64) * 100.0
        } else {
            0.0
        };
        
        // ネットワーク統計情報の集計
        let (network_rx_bytes, network_tx_bytes) = stats.networks
            .as_ref()
            .map(|networks| {
                networks.values().fold((0, 0), |(total_rx, total_tx), network| {
                    (total_rx + network.rx_bytes, total_tx + network.tx_bytes)
                })
            })
            .unwrap_or((0, 0));
        
        // ブロックI/O統計情報の集計
        let (block_read_bytes, block_write_bytes) = if let Some(io_stats) = &stats.blkio_stats.io_service_bytes_recursive {
            io_stats.iter().fold((0, 0), |(read, write), io_stat| {
                match io_stat.op.as_deref() {
                    Some("Read") => (read + io_stat.value.unwrap_or(0), write),
                    Some("Write") => (read, write + io_stat.value.unwrap_or(0)),
                    _ => (read, write),
                }
            })
        } else {
            (0, 0)
        };
        
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
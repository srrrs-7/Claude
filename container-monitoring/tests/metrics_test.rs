use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use mockall::predicate::*;
use mockall::mock;
use tokio::sync::Mutex;

use container_monitoring::docker::{ContainerInfo, ContainerStats, DockerClient};
use container_monitoring::metrics::MetricsCollector;
use container_monitoring::config::MetricsConfig;

// DockerClientのモック作成
mock! {
    pub DockerClientMock {}
    impl Clone for DockerClientMock {
        fn clone(&self) -> Self;
    }
    #[async_trait::async_trait]
    impl DockerClient for DockerClientMock {
        fn new(_config: &container_monitoring::config::DockerConfig) -> Result<Self>;
        async fn list_containers(&self) -> Result<Vec<ContainerInfo>>;
        async fn collect_container_stats(&self, containers: &mut [ContainerInfo]) -> Result<()>;
    }
}

#[tokio::test]
async fn test_metrics_collector_calculate_delta() -> Result<()> {
    // テスト用の設定
    let config = MetricsConfig {
        enable_cpu: true,
        enable_memory: true,
        enable_network: true,
        enable_disk: true,
    };

    // DockerClientのモックを作成
    let mut mock_docker = MockDockerClientMock::new();
    
    // list_containersの振る舞いを設定
    mock_docker
        .expect_list_containers()
        .returning(|| {
            Ok(vec![
                ContainerInfo {
                    id: "container1".to_string(),
                    name: "test_container".to_string(),
                    image: "test_image".to_string(),
                    status: "running".to_string(),
                    stats: None,
                },
            ])
        });
    
    // collect_container_statsの振る舞いを設定
    mock_docker
        .expect_collect_container_stats()
        .returning(|containers| {
            // コンテナにダミーの統計情報を設定
            for container in containers {
                container.stats = Some(ContainerStats {
                    cpu_usage_percent: 10.0,
                    memory_usage_bytes: 1024 * 1024, // 1MB
                    memory_limit_bytes: 1024 * 1024 * 10, // 10MB
                    memory_usage_percent: 10.0,
                    network_rx_bytes: 1000,
                    network_tx_bytes: 500,
                    block_read_bytes: 2000,
                    block_write_bytes: 1000,
                    pids: 5,
                });
            }
            Ok(())
        });

    // MetricsCollectorを作成
    let mut collector = MetricsCollector::new(mock_docker, &config)?;
    
    // プライベートのcalculate_deltaメソッドをテストするためのヘルパー
    let mut prev_values = HashMap::new();
    prev_values.insert("container1".to_string(), 500u64);
    
    // 最初のデルタを計算（1000 - 500 = 500）
    let delta = collector.calculate_delta(&mut prev_values, "container1", 1000);
    assert_eq!(delta, 500);
    assert_eq!(prev_values.get("container1"), Some(&1000));
    
    // 2回目のデルタを計算（1500 - 1000 = 500）
    let delta = collector.calculate_delta(&mut prev_values, "container1", 1500);
    assert_eq!(delta, 500);
    assert_eq!(prev_values.get("container1"), Some(&1500));
    
    // カウンターがリセットされた場合（300 < 1500）
    let delta = collector.calculate_delta(&mut prev_values, "container1", 300);
    assert_eq!(delta, 300); // リセット時は現在値をそのまま返す
    assert_eq!(prev_values.get("container1"), Some(&300));
    
    // 存在しないコンテナのデルタを計算
    let delta = collector.calculate_delta(&mut prev_values, "non_existent", 100);
    assert_eq!(delta, 100); // 初回は現在値をそのまま返す
    assert_eq!(prev_values.get("non_existent"), Some(&100));
    
    Ok(())
}

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use container_monitoring::config::MetricsConfig;
use container_monitoring::docker::{ContainerInfo, ContainerStats};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;

// ベンチマーク用のモックデータ生成関数
fn generate_mock_container_data(count: usize) -> Vec<ContainerInfo> {
    let mut containers = Vec::new();
    
    for i in 0..count {
        let container = ContainerInfo {
            id: format!("container_{}", i),
            name: format!("test_container_{}", i),
            image: "test_image".to_string(),
            status: "running".to_string(),
            stats: Some(ContainerStats {
                cpu_usage_percent: 10.0,
                memory_usage_bytes: 1024 * 1024 * i as u64, // i MB
                memory_limit_bytes: 1024 * 1024 * 100, // 100 MB
                memory_usage_percent: (i as f64) / 100.0 * 100.0,
                network_rx_bytes: 1000 * i as u64,
                network_tx_bytes: 500 * i as u64,
                block_read_bytes: 2000 * i as u64,
                block_write_bytes: 1000 * i as u64,
                pids: i as u64,
            }),
        };
        
        containers.push(container);
    }
    
    containers
}

// デルタ計算のベンチマーク
fn bench_calculate_delta(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    // ベンチマークグループを作成
    let mut group = c.benchmark_group("delta_calculation");
    
    // 各サイズでベンチマークを実行
    for size in [10, 100, 1000].iter() {
        group.bench_function(format!("calculate_delta_{}", size), |b| {
            // ベンチマーク前に一度だけ初期化
            let mut prev_values = HashMap::new();
            
            // *sizeの半分の要素をprev_valuesに初期値として設定
            for i in 0..(size / 2) {
                prev_values.insert(format!("container_{}", i), 500 * i as u64);
            }
            
            let prev_values = Arc::new(Mutex::new(prev_values));
            
            b.to_async(&rt).iter(|| async {
                let mut prev_values = prev_values.lock().await;
                
                let mut results = Vec::new();
                // ランダムなデルタ計算を実行
                for i in 0..*size {
                    let container_id = format!("container_{}", i);
                    let current_value = 1000 * i as u64;
                    
                    let prev_value = prev_values.get(&container_id).copied().unwrap_or(0);
                    
                    // デルタ計算ロジックをコピー
                    prev_values.insert(container_id, current_value);
                    
                    let delta = if current_value >= prev_value {
                        current_value - prev_value
                    } else {
                        current_value
                    };
                    
                    results.push(delta);
                }
                
                black_box(results)
            });
        });
    }
    
    group.finish();
}

// コンテナ処理のベンチマーク
fn bench_process_containers(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    // ベンチマークグループを作成
    let mut group = c.benchmark_group("container_processing");
    
    // 各サイズでベンチマークを実行
    for size in [10, 50, 100].iter() {
        group.bench_function(format!("process_containers_{}", size), |b| {
            let containers = generate_mock_container_data(*size);
            
            b.to_async(&rt).iter(|| async {
                let result = process_containers(containers.clone()).await;
                black_box(result)
            });
        });
    }
    
    group.finish();
}

// コンテナ処理をシミュレートする関数
async fn process_containers(containers: Vec<ContainerInfo>) -> HashMap<String, (f64, u64)> {
    let mut results = HashMap::new();
    
    for container in containers {
        if let Some(stats) = container.stats {
            // CPUとメモリの使用状況を集計
            results.insert(
                container.id,
                (stats.cpu_usage_percent, stats.memory_usage_bytes),
            );
        }
    }
    
    results
}

criterion_group!(benches, bench_calculate_delta, bench_process_containers);
criterion_main!(benches);

# Container Monitoring System with Rust and OpenTelemetry

このプロジェクトは、RustとOpenTelemetryを使用してDocker/Kubernetesコンテナの監視基盤を提供します。vectorを使って効率的にメトリクスを収集・処理します。

## 特徴

- Rustによる高性能な実装
- OpenTelemetryによるメトリクス・トレーシング
- Prometheus互換のメトリクスエンドポイント
- Dockerコンテナの詳細なメトリクス収集
  - CPU使用率
  - メモリ使用率と制限
  - ネットワークI/O
  - ディスクI/O
- Grafanaダッシュボード付き

## アーキテクチャ

```
┌─────────────────┐    ┌───────────────────┐    ┌────────────────┐    ┌─────────────┐
│  コンテナ監視   │    │  OpenTelemetry    │    │                │    │             │
│  アプリケーション ├───►│  Collector        ├───►│  Prometheus   ├───►│  Grafana    │
└─────────────────┘    └───────────────────┘    └────────────────┘    └─────────────┘
        │                                              ▲
        │                                              │
        ▼                                              │
┌─────────────────┐                             ┌──────┴───────┐
│  Docker API     │                             │              │
└─────────────────┘                             │  アラート    │
                                               └──────────────┘
```

## 必要条件

- Rust 1.85以上
- Docker / Docker Compose
- Dockerソケットへのアクセス権限

## 設定

`config/config.toml`ファイルで様々な設定を行えます：

```toml
[general]
# メトリクス収集間隔（秒）
interval = 15

[telemetry]
service_name = "container-monitoring"
otel_exporter = "otlp"
otel_endpoint = "http://otel-collector:4317"
prometheus_port = 8080

[docker]
# Dockerソケットパス
socket_path = "/var/run/docker.sock"

[metrics]
# 特定のメトリクス収集の有効/無効
enable_cpu = true
enable_memory = true
enable_network = true
enable_disk = true

[logging]
level = "info"
```

## 使い方

### Docker Composeで実行

```bash
# ビルドして起動
docker-compose up -d

# ログを確認
docker-compose logs -f container-monitoring

# 停止
docker-compose down
```

### アクセス方法

- Prometheus UI: http://localhost:9090
- Grafana: http://localhost:3000 (ユーザー名: admin, パスワード: admin)
- メトリクスエンドポイント: http://localhost:8080/metrics
- ヘルスチェック: http://localhost:8080/health

## 開発

### ローカルビルド

```bash
# ビルド
cargo build --release

# テスト
cargo test

# 実行（ローカル開発時）
cargo run -- --config config/config.toml
```

## メトリクス

収集されるメトリクスの一部：

- `container_cpu_usage_percent` - コンテナのCPU使用率（%）
- `container_memory_usage_bytes` - メモリ使用量（バイト）
- `container_memory_limit_bytes` - メモリ制限（バイト）
- `container_memory_usage_percent` - メモリ使用率（%）
- `container_network_receive_bytes_total` - ネットワーク受信バイト数（累計）
- `container_network_transmit_bytes_total` - ネットワーク送信バイト数（累計）
- `container_fs_reads_bytes_total` - ディスク読み込みバイト数（累計）
- `container_fs_writes_bytes_total` - ディスク書き込みバイト数（累計）
- `container_count` - コンテナ数（ステータス別）

すべてのメトリクスには以下のラベルが付与されます：
- `container_id`
- `container_name`
- `image`

## ライセンス

MIT

## 貢献

プルリクエストや問題報告は大歓迎です！

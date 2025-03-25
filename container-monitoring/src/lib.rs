// lib.rsは統合テスト用のライブラリとしてプロジェクトの各モジュールをエクスポートします

// 各モジュールを公開
pub mod config;
pub mod docker;
pub mod metrics;
pub mod telemetry;
pub mod server;

// 主要な型やトレイトを再エクスポート
pub use config::{Config, load_config};
pub use docker::{DockerClient, ContainerInfo, ContainerStats};
pub use metrics::MetricsCollector;
pub use telemetry::init_telemetry;
pub use server::start_metrics_server;

// コンテナモニタリングライブラリのバージョン情報
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// ライブラリの初期化関数
pub fn init() {
    println!("Container Monitoring Library v{} initialized", VERSION);
}

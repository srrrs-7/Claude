use anyhow::Result;
use container_monitoring::config::{load_config, Config};
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

#[test]
fn test_load_valid_config() -> Result<()> {
    // テスト用の一時ディレクトリを作成
    let temp_dir = tempdir()?;
    let config_path = temp_dir.path().join("test_config.toml");

    // テスト用の設定ファイルを作成
    let config_content = r#"
    [general]
    interval = 15

    [telemetry]
    service_name = "test-service"
    otel_exporter = "otlp"
    otel_endpoint = "http://localhost:4317"
    prometheus_port = 8080

    [docker]
    socket_path = "/var/run/docker.sock"

    [metrics]
    enable_cpu = true
    enable_memory = true
    enable_network = true
    enable_disk = true

    [logging]
    level = "debug"
    "#;

    let mut file = File::create(&config_path)?;
    file.write_all(config_content.as_bytes())?;

    // 設定ファイルを読み込む
    let config = load_config(&config_path)?;

    // 読み込んだ設定を検証
    assert_eq!(config.general.interval, 15);
    assert_eq!(config.telemetry.service_name, "test-service");
    assert_eq!(config.telemetry.otel_exporter, "otlp");
    assert_eq!(config.telemetry.otel_endpoint, "http://localhost:4317");
    assert_eq!(config.telemetry.prometheus_port, 8080);
    assert_eq!(config.docker.socket_path, "/var/run/docker.sock");
    assert!(config.metrics.enable_cpu);
    assert!(config.metrics.enable_memory);
    assert!(config.metrics.enable_network);
    assert!(config.metrics.enable_disk);
    assert_eq!(config.logging.level, "debug");

    Ok(())
}

#[test]
fn test_load_invalid_config() {
    // テスト用の一時ディレクトリを作成
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("invalid_config.toml");

    // 不正な設定ファイルを作成
    let config_content = r#"
    [general]
    # intervalが欠けている

    [telemetry]
    service_name = "test-service"
    # 他のフィールドが欠けている
    "#;

    let mut file = File::create(&config_path).unwrap();
    file.write_all(config_content.as_bytes()).unwrap();

    // 設定ファイルの読み込みが失敗することを確認
    let result = load_config(&config_path);
    assert!(result.is_err());
}

use pprof::ProfilerGuard;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::time::Duration;
use tracing::{info, warn};

/// CPUプロファイリングを開始し、ProfilerGuardを返します。
/// このGuardがドロップされたとき、プロファイルが生成されます。
pub fn start_cpu_profiling() -> Option<ProfilerGuard<'static>> {
    match pprof::ProfilerGuard::new(100) {
        Ok(guard) => {
            info!("CPU profiling started");
            Some(guard)
        }
        Err(err) => {
            warn!("Failed to start CPU profiling: {}", err);
            None
        }
    }
}

/// 指定した期間だけCPUプロファイリングを実行し、結果をファイルに保存します。
pub async fn profile_for_duration(duration: Duration, output_path: impl AsRef<Path>) -> Result<(), String> {
    let guard = start_cpu_profiling().ok_or_else(|| "Failed to start profiling".to_string())?;
    
    // 指定された期間だけ待機
    tokio::time::sleep(duration).await;
    
    // プロファイリング結果を取得
    let report = guard.report().map_err(|e| format!("Failed to generate report: {}", e))?;
    
    // フレームグラフをSVGで保存
    let file = File::create(output_path.as_ref()).map_err(|e| format!("Failed to create file: {}", e))?;
    report.flamegraph(file).map_err(|e| format!("Failed to write flamegraph: {}", e))?;
    
    info!("CPU profile saved to {:?}", output_path.as_ref());
    Ok(())
}

/// 現在のメモリ使用量をログに記録します。
pub fn log_memory_usage() {
    #[cfg(target_os = "linux")]
    {
        use std::fs::read_to_string;
        
        // LinuxのプロセスメモリStatをパース
        if let Ok(stat) = read_to_string("/proc/self/status") {
            if let Some(vm_line) = stat.lines().find(|line| line.starts_with("VmRSS:")) {
                if let Some(kb_str) = vm_line.split_whitespace().nth(1) {
                    if let Ok(kb) = kb_str.parse::<u64>() {
                        let mb = kb / 1024;
                        info!("Current memory usage: {} MB", mb);
                        return;
                    }
                }
            }
        }
    }
    
    // 非Linuxプラットフォームまたはエラー時
    info!("Memory usage tracking not available on this platform");
}

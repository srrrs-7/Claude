#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod weather;

use weather::{WeatherService, WeatherResponse};
use tauri::State;

// 簡略化した実装ではグローバル状態が不要なのでダミーの構造体を使用
struct AppState;

#[tauri::command]
async fn get_weather(city: String, _state: State<'_, AppState>) -> Result<WeatherResponse, String> {
    // 検索クエリのログ出力
    println!("Searching weather for: {}", city);
    
    // 新しいサービスインスタンスを作成
    let service = WeatherService::new();
    
    // 天気情報を取得して結果を返す
    match service.get_weather(&city).await {
        Ok(data) => {
            println!("Weather data retrieved successfully for {}", city);
            Ok(data)
        },
        Err(e) => {
            let error_msg = format!("Failed to get weather: {}", e.0);
            println!("{}", error_msg);
            Err(error_msg)
        }
    }
}

fn main() {
    tauri::Builder::default()
        .manage(AppState)
        .invoke_handler(tauri::generate_handler![get_weather])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

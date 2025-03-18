use serde::{Deserialize, Serialize};
use chrono::Utc;
use scraper::{Html, Selector};
use std::collections::HashMap;

// シンプルなエラー型
#[derive(Debug, thiserror::Error)]
#[error("Weather error: {0}")]
pub struct WeatherError(pub String);

#[derive(Debug, Serialize, Deserialize)]
pub struct WeatherResponse {
    pub location: String,
    pub temperature: f64,
    pub feels_like: f64,
    pub humidity: u32,
    pub wind_speed: f64,
    pub description: String,
    pub icon: String,
    pub updated_at: String,
    pub forecast: Vec<DayForecast>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DayForecast {
    pub date: String,
    pub day_of_week: String,
    pub high_temp: String,
    pub low_temp: String,
    pub precipitation: String,
    pub description: String,
    pub icon: String,
}

pub struct WeatherService {
    client: reqwest::Client,
}

impl WeatherService {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
    
    pub async fn get_weather(&self, city: &str) -> Result<WeatherResponse, WeatherError> {
        // tenki.jpを使用したスクレイピング
        match self.scrape_tenkijp_weather(city).await {
            Ok(data) => Ok(data),
            Err(e) => {
                // エラーをログに出力
                println!("スクレイピングエラー: {}. モックデータを使用します。", e.0);
                
                // モックデータを返す
                let now = Utc::now();
                Ok(WeatherResponse {
                    location: city.to_string(),
                    temperature: 22.5,
                    feels_like: 21.0,
                    humidity: 65,
                    wind_speed: 5.2,
                    description: "晴れ".to_string(),
                    icon: "https://openweathermap.org/img/wn/01d@2x.png".to_string(),
                    updated_at: now.to_rfc3339(),
                    forecast: vec![],
                })
            }
        }
    }
    
    // tenki.jpからのスクレイピングによる天気情報取得
    async fn scrape_tenkijp_weather(&self, city: &str) -> Result<WeatherResponse, WeatherError> {
        // 都市名から検索URLを決定
        let url = self.get_tenkijp_url(city)?;
        
        println!("スクレイピングURL: {}", url);
        
        // HTTPリクエスト送信
        let response = self.client
            .get(&url)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
            .send()
            .await
            .map_err(|e| WeatherError(format!("リクエストエラー: {}", e)))?;
        
        if !response.status().is_success() {
            return Err(WeatherError(format!("HTTP エラー: {}", response.status())));
        }
        
        let html_content = response.text().await
            .map_err(|e| WeatherError(format!("HTML取得エラー: {}", e)))?;
        
        // HTMLパース
        let document = Html::parse_document(&html_content);
        
        // 現在時刻
        let now = Utc::now();
        
        // 各情報を抽出
        let location_name = self.extract_location(&document, city)?;
        let temperature = self.extract_temperature(&document)?;
        let description = self.extract_weather_description(&document)?;
        let humidity = self.extract_humidity(&document)?;
        let wind_speed = self.extract_wind_speed(&document)?;
        let feels_like = self.estimate_feels_like(temperature, humidity, wind_speed);
        let icon = self.get_weather_icon(&description);
        
        // 週間予報を取得
        let forecast = self.extract_forecast(&document);
        
        Ok(WeatherResponse {
            location: location_name,
            temperature,
            feels_like,
            humidity,
            wind_speed,
            description,
            icon,
            updated_at: now.to_rfc3339(),
            forecast,
        })
    }
    
    // 都市名からtenki.jpのURLを生成
    fn get_tenkijp_url(&self, city: &str) -> Result<String, WeatherError> {
        // 都市コードのマッピング（必要に応じて拡張）
        let city_codes: HashMap<&str, &str> = [
            ("東京", "3/16/4410"),
            ("大阪", "6/30/6200"),
            ("名古屋", "5/26/5110"),
            ("札幌", "1/2/1400"),
            ("福岡", "9/41/8210"),
            ("京都", "6/29/6110"),
            ("神戸", "6/30/6310"),
            ("横浜", "3/17/4610"),
            ("広島", "7/35/6710"),
            ("仙台", "2/10/3410"),
            ("tokyo", "3/16/4410"),
            ("osaka", "6/30/6200"),
            ("nagoya", "5/26/5110"),
            ("sapporo", "1/2/1400"),
            ("fukuoka", "9/41/8210"),
            ("kyoto", "6/29/6110"),
            ("kobe", "6/30/6310"),
            ("yokohama", "3/17/4610"),
            ("hiroshima", "7/35/6710"),
            ("sendai", "2/10/3410"),
        ].iter().cloned().collect();
        
        let city_code = match city_codes.get(city) {
            Some(code) => code,
            None => {
                // デフォルトは東京
                println!("都市コードが見つかりません: {}, 東京を使用します", city);
                "3/16/4410"
            }
        };
        
        Ok(format!("https://tenki.jp/forecast/{}/", city_code))
    }
    
    // 地域名を抽出
    fn extract_location(&self, document: &Html, default_city: &str) -> Result<String, WeatherError> {
        let selector = Selector::parse("h2").ok()
            .ok_or_else(|| WeatherError("ロケーションセレクタのパースに失敗".to_string()))?;
        
        for element in document.select(&selector) {
            let text = element.text().collect::<Vec<_>>().join(" ");
            if text.contains("の天気") {
                // "〜の天気" という形式から地域名を抽出
                return Ok(text.replace("の天気", "").trim().to_string());
            }
        }
        
        // 見つからない場合はデフォルト都市を返す
        Ok(default_city.to_string())
    }
    
    // 気温を抽出
    fn extract_temperature(&self, document: &Html) -> Result<f64, WeatherError> {
        // 現在の気温を抽出するセレクタを試す
        let selectors = [
            ".weather-now__temperature",
            ".current-temp__body",
            ".current-temp",
            ".weather-detail__temp",
            ".today-weather-main__item-temp",
            ".today-weather-main__item-value",
            ".today-temp",
            ".forecast__table tbody tr:first-child td:nth-child(2) strong"
        ];
        
        // 各セレクタを試す
        for selector_str in selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(element) = document.select(&selector).next() {
                    let text = element.text().collect::<Vec<_>>().join("");
                    println!("Found temperature with selector: {}: {}", selector_str, text);
                    
                    // 数字部分のみを抽出して変換
                    let temp_str = text.replace("°", "").replace("℃", "").replace("\n", "").trim().to_string();
                    if let Ok(temp) = temp_str.parse::<f64>() {
                        return Ok(temp);
                    }
                }
            }
        }
        
        // 代替手段: 予報から今日の気温を取得
        if let Ok(selector) = Selector::parse(".forecast-point-temp") {
            if let Some(element) = document.select(&selector).next() {
                let text = element.text().collect::<Vec<_>>().join("");
                println!("Found forecast temperature: {}", text);
                
                let temp_str = text.replace("°", "").replace("℃", "").replace("\n", "").trim().to_string();
                if let Ok(temp) = temp_str.parse::<f64>() {
                    return Ok(temp);
                }
            }
        }
        
        // デフォルト値を返す
        println!("Could not find temperature, using default");
        Ok(20.0)
    }
    
    // 天気の説明を抽出
    fn extract_weather_description(&self, document: &Html) -> Result<String, WeatherError> {
        // 複数のセレクタを試す
        let selectors = [
            ".weather-now__status",
            ".weather-now__text",
            ".weather__main-telop",
            ".current-weather-telop",
            ".today-weather-main__item-weather",
            ".forecast__table tbody tr:first-child td:nth-child(1) p",
            ".weather-detail__text"
        ];
        
        for selector_str in selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(element) = document.select(&selector).next() {
                    let text = element.text().collect::<Vec<_>>().join("");
                    let cleaned = text.trim().to_string();
                    if !cleaned.is_empty() {
                        println!("Found weather description with selector: {}: {}", selector_str, cleaned);
                        return Ok(cleaned);
                    }
                }
            }
        }
        
        // デフォルト値を返す
        println!("Could not find weather description, using default");
        Ok("曇り".to_string())
    }
    
    // 湿度を抽出
    fn extract_humidity(&self, document: &Html) -> Result<u32, WeatherError> {
        // 複数のセレクタを試す
        let selectors = [
            ".weather-now__precipitation",
            ".precip",
            ".precipitation",
            ".weather-detail__humidity",
            ".humidity",
            "[data-humidity]"
        ];
        
        for selector_str in selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(element) = document.select(&selector).next() {
                    let text = element.text().collect::<Vec<_>>().join("");
                    println!("Found humidity with selector: {}: {}", selector_str, text);
                    
                    // 湿度を抽出 (通常は "湿度XX%" の形式)
                    if let Some(humidity_str) = text.split_whitespace()
                        .find(|s| s.contains('%'))
                        .map(|s| s.replace("%", "")) {
                        
                        if let Ok(humidity) = humidity_str.parse::<u32>() {
                            return Ok(humidity);
                        }
                    }
                    
                    // 単純な数字だけの場合も試す
                    for word in text.split_whitespace() {
                        if let Ok(humidity) = word.replace("%", "").parse::<u32>() {
                            if humidity <= 100 { // 合理的な湿度の範囲かチェック
                                return Ok(humidity);
                            }
                        }
                    }
                }
            }
        }
        
        // デフォルト値を返す
        println!("Could not find humidity, using default");
        Ok(60)
    }
    
    // 風速を抽出
    fn extract_wind_speed(&self, document: &Html) -> Result<f64, WeatherError> {
        // 複数のセレクタを試す
        let selectors = [
            ".weather-now__wind-speed",
            ".wind-speed",
            ".weather-detail__wind",
            ".windSpeed",
            "[data-wind]",
            ".today-weather-main__item-wind"
        ];
        
        for selector_str in selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(element) = document.select(&selector).next() {
                    let text = element.text().collect::<Vec<_>>().join("");
                    println!("Found wind speed with selector: {}: {}", selector_str, text);
                    
                    // 風速を抽出 (通常は "XX m/s" の形式)
                    for word in text.split_whitespace() {
                        // 数字だけを取り出す
                        if let Ok(speed) = word.replace("m/s", "").trim().parse::<f64>() {
                            if speed < 100.0 { // 合理的な風速かチェック
                                return Ok(speed);
                            }
                        }
                    }
                }
            }
        }
        
        // デフォルト値を返す
        println!("Could not find wind speed, using default");
        Ok(3.0)
    }
    
    // 体感温度を計算 (気温、湿度、風速から推定)
    fn estimate_feels_like(&self, temp: f64, humidity: u32, wind_speed: f64) -> f64 {
        // 簡易的な体感温度計算（実際にはもっと複雑な計算式があります）
        if temp > 25.0 {
            // 暑い日は湿度の影響が大きい
            temp + (humidity as f64 - 60.0) * 0.1 - wind_speed * 0.2
        } else if temp < 10.0 {
            // 寒い日は風速の影響が大きい
            temp - wind_speed * 0.5
        } else {
            // 普通の日
            temp - wind_speed * 0.3
        }
    }
    
    // 予報情報を抽出
    fn extract_forecast(&self, document: &Html) -> Vec<DayForecast> {
        let mut forecasts = Vec::new();
        
        // 日付セレクタ
        if let Ok(date_selector) = Selector::parse(".forecast-days-wrap .forecast-date-h") {
            // 天気セレクタ
            if let Ok(weather_selector) = Selector::parse(".forecast-days-wrap .forecast-telop") {
                // 気温セレクタ
                if let Ok(temp_selector) = Selector::parse(".forecast-days-wrap .forecast-top__temp") {
                    
                    let dates: Vec<_> = document.select(&date_selector).collect();
                    let weather_elements: Vec<_> = document.select(&weather_selector).collect();
                    let temp_elements: Vec<_> = document.select(&temp_selector).collect();
                    
                    let count = dates.len().min(weather_elements.len()).min(temp_elements.len());
                    
                    for i in 0..count {
                        let date_text = dates[i].text().collect::<Vec<_>>().join(" ");
                        let parts: Vec<&str> = date_text.split_whitespace().collect();
                        
                        let date = if parts.len() > 0 { parts[0].to_string() } else { "不明".to_string() };
                        let day_of_week = if parts.len() > 1 { parts[1].to_string() } else { "".to_string() };
                        
                        let description = weather_elements[i].text().collect::<Vec<_>>().join("").trim().to_string();
                        
                        let temp_text = temp_elements[i].text().collect::<Vec<_>>().join("");
                        let temp_parts: Vec<&str> = temp_text.split("/").collect();
                        
                        let high_temp = if temp_parts.len() > 0 { 
                            temp_parts[0].trim().to_string() 
                        } else { 
                            "N/A".to_string() 
                        };
                        
                        let low_temp = if temp_parts.len() > 1 { 
                            temp_parts[1].trim().to_string() 
                        } else { 
                            "N/A".to_string() 
                        };
                        
                        forecasts.push(DayForecast {
                            date,
                            day_of_week,
                            high_temp,
                            low_temp,
                            precipitation: "0%".to_string(), // 詳細なデータ抽出が難しい場合はデフォルト値
                            description: description.clone(),
                            icon: self.get_weather_icon(&description),
                        });
                    }
                }
            }
        }
        
        forecasts
    }
    
    // 天気アイコンのURLを取得
    fn get_weather_icon(&self, description: &str) -> String {
        // 天気の説明に基づいてアイコンを選択
        let icon_code = match description {
            d if d.contains("晴") && d.contains("曇") => "02d", // 晴れ時々曇り
            d if d.contains("晴") && d.contains("雨") => "10d", // 晴れ時々雨
            d if d.contains("晴") => "01d",                    // 晴れ
            d if d.contains("曇") && d.contains("雨") => "09d", // 曇り時々雨
            d if d.contains("曇") => "03d",                    // 曇り
            d if d.contains("雨") && d.contains("雪") => "13d", // 雨または雪
            d if d.contains("雨") && d.contains("強") => "11d", // 強い雨
            d if d.contains("雨") => "09d",                    // 雨
            d if d.contains("雪") => "13d",                    // 雪
            d if d.contains("雷") => "11d",                    // 雷
            d if d.contains("霧") => "50d",                    // 霧
            _ => "01d",  // デフォルトは晴れ
        };
        
        format!("https://openweathermap.org/img/wn/{}@2x.png", icon_code)
    }
}

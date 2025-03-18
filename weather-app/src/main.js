const { invoke } = window.__TAURI__.tauri;

document.addEventListener('DOMContentLoaded', () => {
  const cityInput = document.getElementById('city-input');
  const searchBtn = document.getElementById('search-btn');
  const errorMessage = document.getElementById('error-message');
  const weatherContainer = document.getElementById('weather-container');
  const loadingSpinner = document.getElementById('loading-spinner');
  
  const locationElement = document.getElementById('location');
  const updatedAtElement = document.getElementById('updated-at');
  const weatherIconElement = document.getElementById('weather-icon');
  const descriptionElement = document.getElementById('description');
  const temperatureElement = document.getElementById('temperature');
  const feelsLikeElement = document.getElementById('feels-like');
  const humidityElement = document.getElementById('humidity');
  const windSpeedElement = document.getElementById('wind-speed');
  
  // 日付フォーマット関数
  const formatDate = (dateString) => {
    const date = new Date(dateString);
    const options = { 
      year: 'numeric', 
      month: 'short', 
      day: 'numeric', 
      hour: '2-digit', 
      minute: '2-digit' 
    };
    return date.toLocaleDateString('ja-JP', options);
  };
  
  // 天気アイコンのクラス取得関数（FontAwesome用）
  const getWeatherIconClass = (description) => {
    const desc = description.toLowerCase();
    
    if (desc.includes('sunny') || desc.includes('clear') || desc.includes('晴れ')) {
      return 'fas fa-sun';
    } else if (desc.includes('partly cloudy') || desc.includes('一部曇り')) {
      return 'fas fa-cloud-sun';
    } else if (desc.includes('cloudy') || desc.includes('曇り')) {
      return 'fas fa-cloud';
    } else if (desc.includes('rain') || desc.includes('rainy') || desc.includes('雨')) {
      return 'fas fa-cloud-rain';
    } else if (desc.includes('thunderstorm') || desc.includes('雷雨')) {
      return 'fas fa-bolt';
    } else if (desc.includes('snow') || desc.includes('snowy') || desc.includes('雪')) {
      return 'fas fa-snowflake';
    } else if (desc.includes('mist') || desc.includes('fog') || desc.includes('霧')) {
      return 'fas fa-smog';
    }
    
    return 'fas fa-sun'; // デフォルト
  };
  
  // 天気情報の取得関数
  const getWeather = async (city) => {
    try {
      showLoading(true);
      hideError();
      hideWeatherData();
      
      console.log(`${city}の天気情報を取得中...`);
      const weatherData = await invoke('get_weather', { city });
      console.log('取得した天気データ:', weatherData);
      
      // UI更新
      locationElement.textContent = weatherData.location;
      updatedAtElement.textContent = `最終更新: ${formatDate(weatherData.updated_at)}`;
      weatherIconElement.src = weatherData.icon;
      
      // 日本語と英語の両方に対応
      descriptionElement.textContent = weatherData.description;
      
      temperatureElement.textContent = `${Math.round(weatherData.temperature)}°C`;
      feelsLikeElement.textContent = `体感温度: ${Math.round(weatherData.feels_like)}°C`;
      humidityElement.textContent = `${weatherData.humidity}%`;
      windSpeedElement.textContent = `${weatherData.wind_speed} m/s`;
      
      // 予報データがあれば表示
      if (weatherData.forecast && weatherData.forecast.length > 0) {
        renderForecast(weatherData.forecast);
      }
      
      showWeatherData();
    } catch (error) {
      console.error('エラーが発生しました:', error);
      showError(`天気情報の取得に失敗しました: ${error}`);
    } finally {
      showLoading(false);
    }
  };
  
  // 予報データをレンダリング
  const renderForecast = (forecast) => {
    const forecastContainer = document.getElementById('forecast-items');
    forecastContainer.innerHTML = ''; // 一度クリア
    
    forecast.forEach((day, index) => {
      const forecastItem = document.createElement('div');
      forecastItem.className = 'forecast-item';
      forecastItem.style.setProperty('--i', index); // アニメーションの遅延用
      
      // 曜日と日付
      const dayElem = document.createElement('div');
      dayElem.className = 'forecast-day';
      dayElem.textContent = day.day_of_week || '?';
      
      const dateElem = document.createElement('div');
      dateElem.className = 'forecast-date';
      dateElem.textContent = day.date || '';
      
      // 天気アイコン
      const iconElem = document.createElement('img');
      iconElem.className = 'forecast-icon';
      iconElem.src = day.icon;
      iconElem.alt = day.description;
      
      // 天気の説明
      const descElem = document.createElement('div');
      descElem.className = 'forecast-description';
      descElem.textContent = day.description;
      
      // 気温（最高・最低）
      const tempElem = document.createElement('div');
      tempElem.className = 'forecast-temp';
      
      const highTemp = document.createElement('span');
      highTemp.className = 'forecast-temp-high';
      highTemp.textContent = day.high_temp;
      
      const tempSeparator = document.createElement('span');
      tempSeparator.textContent = '/';
      
      const lowTemp = document.createElement('span');
      lowTemp.className = 'forecast-temp-low';
      lowTemp.textContent = day.low_temp;
      
      // 要素を組み立てる
      tempElem.appendChild(highTemp);
      tempElem.appendChild(tempSeparator);
      tempElem.appendChild(lowTemp);
      
      forecastItem.appendChild(dayElem);
      forecastItem.appendChild(dateElem);
      forecastItem.appendChild(iconElem);
      forecastItem.appendChild(descElem);
      forecastItem.appendChild(tempElem);
      
      forecastContainer.appendChild(forecastItem);
    });
  };
  
  // エラーメッセージの表示関数
  const showError = (message) => {
    errorMessage.textContent = message;
    errorMessage.style.display = 'block';
    
    // アニメーションをリセットして再適用
    errorMessage.classList.remove('shake');
    void errorMessage.offsetWidth; // リフローを強制して再アニメーション
    errorMessage.classList.add('shake');
    
    // エラーメッセージを時間経過後に非表示
    setTimeout(() => {
      // フェードアウトアニメーション
      errorMessage.style.animation = 'fadeOut 0.5s ease-in forwards';
      setTimeout(() => {
        hideError();
        errorMessage.style.animation = ''; // アニメーションをリセット
      }, 500);
    }, 4000);
  };
  
  const hideError = () => {
    errorMessage.style.display = 'none';
  };
  
  // ウェザーデータの表示関数
  const showWeatherData = () => {
    weatherContainer.classList.remove('hidden');
    
    // 要素にアニメーションクラスを追加
    setTimeout(() => {
      // アニメーションをリセットするために一度クラスを削除して再度追加
      locationElement.classList.remove('fadeInUp');
      updatedAtElement.classList.remove('fadeInUp');
      
      void locationElement.offsetWidth; // リフローを強制
      
      locationElement.classList.add('fadeInUp');
      updatedAtElement.classList.add('fadeInUp');
    }, 100);
  };
  
  const hideWeatherData = () => {
    weatherContainer.classList.add('hidden');
  };
  
  const showLoading = (isLoading) => {
    if (isLoading) {
      loadingSpinner.classList.remove('hidden');
      loadingSpinner.style.animation = 'spin 1s linear infinite, fadeIn 0.3s ease-out';
    } else {
      // フェードアウトアニメーションを追加
      loadingSpinner.style.animation = 'spin 1s linear infinite, fadeOut 0.3s ease-in forwards';
      setTimeout(() => {
        loadingSpinner.classList.add('hidden');
      }, 300);
    }
  };
  
  // イベントリスナー
  searchBtn.addEventListener('click', () => {
    const city = cityInput.value.trim();
    if (city) {
      getWeather(city);
    } else {
      showError('都市名を入力してください');
    }
  });
  
  cityInput.addEventListener('keypress', (e) => {
    if (e.key === 'Enter') {
      const city = cityInput.value.trim();
      if (city) {
        getWeather(city);
      } else {
        showError('都市名を入力してください');
      }
    }
  });
  
  // 検索ボタンの波紋エフェクト
  searchBtn.addEventListener('mousedown', function(e) {
    // 波紋効果の位置を調整
    const rect = this.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;
    
    // 波紋エフェクトのスタイルをカスタマイズ
    const ripple = document.createElement('span');
    ripple.classList.add('ripple-effect');
    ripple.style.left = `${x}px`;
    ripple.style.top = `${y}px`;
    
    this.appendChild(ripple);
    
    // エフェクト要素を削除
    setTimeout(() => {
      ripple.remove();
    }, 600);
  });
  
  // 初期都市を設定（オプション）
  const setInitialCity = () => {
    const defaultCity = "東京"; // デフォルト都市
    cityInput.value = defaultCity;
    getWeather(defaultCity);
  };
  
  // ページ読み込み時にデフォルト都市の天気を表示
  setInitialCity();
});

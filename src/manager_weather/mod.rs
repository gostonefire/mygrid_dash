use chrono::Local;
use crate::errors::WeatherError;
use crate::models::DataItem;
use crate::traits::Weather;

/// Builder for a new instance of ShellyWeather
/// 
pub struct WeatherBuilder {
    api_key: Option<String>,
}
impl WeatherBuilder {
    
    pub fn new() -> Self {
        Self { api_key: None }
    }
    
    /// Adds api key to builder
    /// 
    /// # Arguments
    /// 
    /// * 'key' - the api key
    pub fn api_key(self, key: String) -> Self {
        Self { api_key: Some(key) }
    }
    
    /// Returns an instance of ShellyWeather if all necessary build steps has been conducted
    /// 
    pub fn build(self) -> Result<impl Weather, WeatherError> {
        if let Some(api_key) = self.api_key {
            Ok(ShellyWeather { api_key, temperature: 0.0 })
        } else {
            Err("missing api key")?
        }
    }
}

/// ShellyWeather manager
/// 
pub struct ShellyWeather {
    api_key: String,
    temperature: f64,
}

impl Weather for ShellyWeather {

    fn get_temperature(&self) -> Result<DataItem<f64>, WeatherError> {
        Ok(DataItem { x: Local::now(), y: 18.5 })
    }
}

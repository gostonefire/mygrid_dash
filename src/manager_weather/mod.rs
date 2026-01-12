pub mod errors;
mod models;

use std::time::Duration;
use chrono::{DateTime, Utc};
use reqwest::Client;
use crate::manager_weather::errors::WeatherError;
use crate::manager_weather::models::{ForecastRecord, MinMax, Temperature};
use crate::models::{DataItem, ForecastData, TemperatureData};


/// Weather manager
/// 
pub struct Weather {
    client: Client,
    host: String,
    sensor: String,
}

impl Weather {

    /// Returns a new instance of Weather
    /// 
    /// # Arguments
    /// 
    /// * 'host' - host running the weather logger service
    /// * 'sensor' - name of sensor to get weather data for
    pub fn new(host: &str, sensor: &str) -> Result<Self, WeatherError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;
        
        Ok(Self { client, host: host.to_string(), sensor: sensor.to_string() })
    }

    /// Returns the temperature forecast from within the given time boundaries
    ///
    /// # Arguments
    ///
    /// * 'from' - from datetime
    /// * 'to' - to datetime (non-inclusive)
    pub async fn get_forecast(&self, from: DateTime<Utc>, to: DateTime<Utc>) -> Result<ForecastData, WeatherError> {
        let url = format!("http://{}/forecast", self.host);

        let req = self.client.get(&url)
            .query(&[("id", "smhi"), ("from", &from.to_rfc3339()), ("to", &to.to_rfc3339())])
            .send().await?;

        let status = req.status();
        if !status.is_success() {
            return Err(WeatherError(format!("{:?}", status)));
        }

        let json = req.text().await?;
        let weather_res: Vec<ForecastRecord> = serde_json::from_str(&json)?;

        let mut result = ForecastData { forecast_temp: Vec::new(), symbol_code: Vec::new() };
        weather_res.into_iter().for_each(|w| {
            w.temperature.map(|t| result.forecast_temp.push(DataItem{x: w.date_time, y: t}));
            w.symbol_code.map(|c| result.symbol_code.push(DataItem{x: w.date_time, y: c}));
        });

        Ok(result)
    }
    
    /// Returns the temperature history from within the given time boundaries
    /// 
    /// # Arguments
    /// 
    /// * 'from' - from datetime
    /// * 'to' - to datetime (non-inclusive)
    /// * 'ensure_from' - if true the 'from' date will have a data item
    pub async fn get_temp_history(&self, from: DateTime<Utc>, to: DateTime<Utc>, ensure_from: bool) -> Result<TemperatureData<f64>, WeatherError> {
        let url = format!("http://{}/temperature", self.host);

        let req = self.client.get(&url)
            .query(&[("id", &self.sensor), ("from", &from.to_rfc3339()), ("to", &to.to_rfc3339())])
            .send().await?;

        let status = req.status();
        if !status.is_success() {
            return Err(WeatherError(format!("{:?}", status)));
        }

        let json = req.text().await?;
        let weather_res: Temperature<f64> = serde_json::from_str(&json)?;
        let from_date = if ensure_from {Some(from)} else {None};
        
        Ok(transform_history(weather_res, from_date, to))
    }
    
    /// Returns today's and yesterday's min/max temperatures
    ///
    /// # Arguments
    ///
    /// * 'from' - from date time
    /// * 'to' - to date time (non-inclusive)
    pub async fn get_min_max(&self, from: DateTime<Utc>, to: DateTime<Utc>) -> Result<(f64, f64), WeatherError> {
        let url = format!("http://{}/minmax", self.host);

        let req = self.client.get(&url)
            .query(&[("id", &self.sensor),("from", &from.to_rfc3339()),("to", &to.to_rfc3339())])
            .send().await?;

        let status = req.status();
        if !status.is_success() {
            return Err(WeatherError(format!("{:?}", status)));
        }

        let json = req.text().await?;
        let minmax: MinMax<f64> = serde_json::from_str(&json)?;
       
        Ok((minmax.min, minmax.max))
    }
}

/// Transforms the history from the weather database to a mygrid dash data model
///
/// While doing so the transformation also ensures that the 'to' date has a data item, and
/// possibly also the 'from' date
/// 
/// # Arguments
/// 
/// * 'history' - the history data to transform
/// * 'from' - optional from date to include with a data item
/// * 'to' - to date to include with a data item
fn transform_history<T: Copy>(history: Temperature<T>, from: Option<DateTime<Utc>>, to: DateTime<Utc>) -> TemperatureData<T> {
    let mut result = TemperatureData {
        history: Vec::new(),
        current_temp: history.current_temp,
        perceived_temp: history.perceived_temp,
    };
    
    if history.history.len() == 0 {
        result
    } else {
        history.history.into_iter().for_each(|w| {result.history.push(DataItem{x: w.x, y: w.y});});
        
        if let Some(from) = from { 
            if result.history[0].x != from {
                    result.history.insert(0, DataItem{x: from, y: result.history[0].y});
            }
        }

        let last = result.history.len() - 1;
        if result.history[last].x != to {
            result.history.push(DataItem{x: to, y: result.history[last].y});
        }

        result
    }
}
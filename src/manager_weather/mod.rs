pub mod errors;
mod models;

use std::time::Duration;
use chrono::{DateTime, Local};
use reqwest::Client;
use crate::manager_weather::errors::WeatherError;
use crate::manager_weather::models::WeatherItem;
use crate::models::DataItem;


/// Weather manager
/// 
pub struct Weather {
    client: Client,
    host: String,
}

impl Weather {

    /// Returns a new instance of Weather
    /// 
    pub fn new(host: &str) -> Result<Self, WeatherError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;
        
        Ok(Self { client, host: host.to_string() })
    }
    
    /// Returns the temperature history from within the given time boundaries (inclusive)
    /// 
    /// # Arguments
    /// 
    /// * 'from' - from datetime
    /// * 'to' - to datetime
    /// * 'ensure_from' - if true the 'from' date will have a data item
    pub async fn get_temp_history(&self, from: DateTime<Local>, to: DateTime<Local>, ensure_from: bool) -> Result<Vec<DataItem<f64>>, WeatherError> {
        let url = format!("http://{}/temperature", self.host);
        
        let req = self.client.get(&url)
            .query(&[("from", &from.to_rfc3339()), ("to", &to.to_rfc3339())])
            .send().await?;

        let status = req.status();
        if !status.is_success() {
            return Err(WeatherError(format!("{:?}", status)));
        }

        let json = req.text().await?;
        let weather_res: Vec<WeatherItem<f64>> = serde_json::from_str(&json)?;
        let from_date = if ensure_from {Some(from)} else {None};
        
        Ok(transform_history(weather_res, from_date, to))
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
fn transform_history<T: Copy>(history: Vec<WeatherItem<T>>, from: Option<DateTime<Local>>, to: DateTime<Local>) -> Vec<DataItem<T>> {
    let mut result: Vec<DataItem<T>> = Vec::new();
    
    if history.len() == 0 {
        result
    } else {
        history.into_iter().for_each(|w| {result.push(DataItem{x: w.x, y: w.y});});
        
        if let Some(from) = from { 
            if result[0].x != from {
                    result.insert(0, DataItem{x: from, y: result[0].y});
            }
        }

        let last = result.len() - 1;
        if result[last].x != to {
            result.push(DataItem{x: to, y: result[last].y});
        }
        
        result
    }
}
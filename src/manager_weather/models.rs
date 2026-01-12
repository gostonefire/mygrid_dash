use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct WeatherItem<T> {
    pub x: DateTime<Utc>,
    pub y: T,
}

#[derive(Deserialize)]
pub struct MinMax<T> {
    pub min: T,
    pub max: T,
}

#[derive(Deserialize)]
pub struct Temperature<T> {
    pub history: Vec<WeatherItem<T>>,
    pub current_temp: Option<T>,
    pub perceived_temp: Option<T>,
}

#[derive(Deserialize)]
pub struct ForecastRecord {
    pub date_time: DateTime<Utc>,
    pub temperature: Option<f64>,
    pub symbol_code: Option<u8>,
}
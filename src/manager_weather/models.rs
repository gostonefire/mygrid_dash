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
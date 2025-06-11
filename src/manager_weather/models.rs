use chrono::{DateTime, Local};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct WeatherItem<T> {
    pub x: DateTime<Local>,
    pub y: T,
}

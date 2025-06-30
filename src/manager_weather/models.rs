use chrono::{DateTime, Local};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct WeatherItem<T> {
    pub x: DateTime<Local>,
    pub y: T,
}

#[derive(Deserialize)]
pub struct TwoDaysMinMax<T> {
    pub yesterday_min: T,
    pub yesterday_max: T,
    pub today_min: T,
    pub today_max: T,
}
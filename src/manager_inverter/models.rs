use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct DataRecord<T> {
    pub data: T,
}

#[derive(Deserialize)]
pub struct Samples {
    #[serde(with = "chrono::serde::ts_seconds")]
    pub ts: DateTime<Utc>,
    pub production: f64,
    pub consumption: f64,
    pub batt_soc: f64,
}

#[derive(Deserialize)]
pub struct HistoryRecord {
    pub samples: Vec<Samples>,
}

#[derive(Deserialize)]
pub struct EnergyIntervals {
    #[serde(with = "chrono::serde::ts_seconds")]
    pub from_ts: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub to_ts: DateTime<Utc>,
    pub feed_in_energy: f64,
    pub grid_consumption_energy: f64,
}

#[derive(Deserialize)]
pub struct EnergyIntervalsRecord {
    pub intervals: Vec<EnergyIntervals>,
}
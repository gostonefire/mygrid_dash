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

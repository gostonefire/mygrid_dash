use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct ForecastValues {
    pub valid_time: DateTime<Local>,
    pub temp: f64,
    pub cloud_factor: f64,
}

#[derive(Deserialize)]
pub struct ProductionValues {
    pub valid_time: DateTime<Local>,
    pub power: f64
}

#[derive(Deserialize)]
pub struct ConsumptionValues {
    pub valid_time: DateTime<Local>,
    pub power: f64
}

#[derive(Deserialize)]
pub struct TariffValues {
    pub valid_time: DateTime<Local>,
    pub buy: f64,
    pub sell: f64,
}

#[derive(Deserialize)]
pub struct BaseData {
    pub date_time: DateTime<Local>,
    pub forecast: Vec<ForecastValues>,
    pub production: Vec<ProductionValues>,
    pub consumption: Vec<ConsumptionValues>,
    pub tariffs: Vec<TariffValues>,
}

#[derive(Deserialize)]
pub struct Block {
    pub block_type: String,
    pub start_time: DateTime<Local>,
    pub end_time: DateTime<Local>,
    pub soc_in: usize,
    pub soc_out: usize,
    pub status: String,
}
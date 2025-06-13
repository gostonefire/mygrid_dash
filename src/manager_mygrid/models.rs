use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};


#[derive(Deserialize)]
pub struct ForecastValues {
    #[serde(rename(deserialize = "valid_time"))]
    pub date_time: DateTime<Local>,
    pub temp: f64,
    pub cloud_factor: f64,
}

#[derive(Deserialize)]
pub struct ProductionValues {
    #[serde(rename(deserialize = "valid_time"))]
    pub date_time: DateTime<Local>,
    pub power: f64
}


#[derive(Deserialize)]
pub struct ConsumptionValues {
    #[serde(rename(deserialize = "valid_time"))]
    pub date_time: DateTime<Local>,
    pub power: f64
}

#[derive(Deserialize)]
pub struct TariffValues {
    #[serde(rename(deserialize = "valid_time"))]
    pub date_time: DateTime<Local>,
    pub buy: f64,
    pub sell: f64,
}

#[derive(Deserialize)]
pub struct BaseData {
    pub forecast: Vec<ForecastValues>,
    pub production_kw: Vec<ProductionValues>,
    pub consumption: Vec<ConsumptionValues>,
    pub tariffs: Vec<TariffValues>,
}

/// Available block types
#[derive(PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockType {
    Charge,
    Hold,
    Use,
}

#[derive(Serialize, Deserialize)]
pub struct Block {
    pub block_type: BlockType,
    pub start_time: DateTime<Local>,
    pub end_time: DateTime<Local>,
    pub soc_in: usize,
    pub soc_out: usize,
    pub status: String,
}



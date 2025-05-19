use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

pub trait ValidDate {
    fn date(&self) -> DateTime<Local>;
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ForecastValues {
    #[serde(rename(deserialize = "valid_time"))]
    pub date_time: DateTime<Local>,
    pub temp: f64,
    pub cloud_factor: f64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ProductionValues {
    #[serde(rename(deserialize = "valid_time"))]
    pub date_time: DateTime<Local>,
    pub power: f64
}


#[derive(Serialize, Deserialize, Clone)]
pub struct ConsumptionValues {
    #[serde(rename(deserialize = "valid_time"))]
    pub date_time: DateTime<Local>,
    pub power: f64
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TariffValues {
    #[serde(rename(deserialize = "valid_time"))]
    pub date_time: DateTime<Local>,
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

#[derive(Serialize, Deserialize)]
pub struct Block {
    pub block_type: String,
    pub start_time: DateTime<Local>,
    pub end_time: DateTime<Local>,
    pub soc_in: usize,
    pub soc_out: usize,
    pub status: String,
}


impl ValidDate for ForecastValues {
    fn date(&self) -> DateTime<Local> {
        self.date_time
    }
}
impl ValidDate for ProductionValues {
    fn date(&self) -> DateTime<Local> {
        self.date_time
    }
}
impl ValidDate for ConsumptionValues {
    fn date(&self) -> DateTime<Local> {
        self.date_time
    }
}
impl ValidDate for TariffValues {
    fn date(&self) -> DateTime<Local> {
        self.date_time
    }
}

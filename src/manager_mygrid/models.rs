use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use crate::manager_mygrid::errors::MyGridError;

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

#[derive(Serialize)]
pub struct Forecast {
    pub date_time: Vec<DateTime<Local>>,
    pub temp: Vec<f64>,
    pub cloud_factor: Vec<f64>,
}
#[derive(Serialize)]
pub struct Production {
    pub date_time: Vec<DateTime<Local>>,
    pub power: Vec<f64>,
}
#[derive(Serialize)]
pub struct Consumption {
    pub date_time: Vec<DateTime<Local>>,
    pub power: Vec<f64>,
}
#[derive(Serialize)]
pub struct Tariffs {
    pub date_time: Vec<DateTime<Local>>,
    pub buy: Vec<f64>,
    pub sell: Vec<f64>,
}

pub struct MygridData {
    pub date_time: DateTime<Local>,
    pub forecast: Forecast,
    pub production: Production,
    pub consumption: Consumption,
    pub tariffs: Tariffs,
}

pub trait Mygrid {
    type Item;
    
    /// Filters and keeps records within the given from and to (non-inclusive)
    ///
    /// # Arguments
    ///
    /// * 'from' - from date time
    /// * 'to' - to date time (non-inclusive)
    fn keep(&self, from: DateTime<Local>, to: DateTime<Local>) -> Self::Item;

    /// Appends a tail of records
    ///
    /// # Arguments
    ///
    /// * 'other' - the struct from where to fetch records to append
    fn append_tail(self, other: &mut Self::Item) -> Self::Item;

    /// Pads (left) with missing dates from midnight and zero for data fields
    ///
    fn pad(self) -> Result<Self::Item, MyGridError>;
}


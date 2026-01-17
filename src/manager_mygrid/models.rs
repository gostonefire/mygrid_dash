use std::fmt;
use std::fmt::Formatter;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};


#[derive(Deserialize)]
pub struct ForecastValues {
    #[serde(rename(deserialize = "valid_time"))]
    pub date_time: DateTime<Utc>,
    pub temp: f64,
    pub cloud_factor: f64,
}

#[derive(Deserialize)]
pub struct ProductionValues {
    #[serde(rename(deserialize = "valid_time"))]
    pub date_time: DateTime<Utc>,
    pub data: f64
}


#[derive(Deserialize)]
pub struct ConsumptionValues {
    #[serde(rename(deserialize = "valid_time"))]
    pub date_time: DateTime<Utc>,
    pub data: f64
}

#[derive(Deserialize)]
pub struct TariffFees {
    // Power grid fees (öre/kWh, exl. VAT)
    pub variable_fee: f64,
    pub spot_fee_percentage: f64,
    pub energy_tax: f64,

    // Electricity trading fees  (öre/kWh, exl. VAT)
    pub swedish_power_grid: f64,
    pub balance_responsibility: f64,
    pub electric_certificate: f64,
    pub guarantees_of_origin: f64,
    pub fixed: f64,
}

#[derive(Deserialize)]
pub struct BaseData {
    pub base_cost: f64,
    pub schedule_cost: f64,
    pub forecast: Vec<ForecastValues>,
    pub production: Vec<ProductionValues>,
    pub consumption: Vec<ConsumptionValues>,
    pub tariff_fees: TariffFees,
}

/// Available block types
#[derive(PartialEq, Eq, Serialize, Deserialize, Clone)]
pub enum BlockType {
    Charge,
    Hold,
    Use,
}

/// Block status
#[derive(Deserialize, Clone)]
pub enum Status {
    Waiting,
    Started,
    Full(usize),
    Error,
}

/// Implementation of the Display Trait for pretty print
impl fmt::Display for Status {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Status::Waiting => write!(f, "Waiting  "),
            Status::Started => write!(f, "Started  "),
            Status::Full(soc) => write!(f, "Full: {:>3}", soc),
            Status::Error   => write!(f, "Error    "),
        }
    }
}

#[derive(Deserialize)]
pub struct SourceBlock {
    pub block_type: BlockType,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub cost: f64,
    pub true_soc_in: Option<usize>,
    pub soc_in: usize,
    pub soc_out: usize,
    pub status: Status,
}

#[derive(Serialize)]
pub struct Block {
    pub block_type: BlockType,
    pub cost: String,
    pub true_soc_in: Option<usize>,
    pub soc_in: usize,
    pub soc_out: usize,
    pub status: String,
    #[serde(default)]
    pub start: String,
    #[serde(skip)]
    pub start_time: DateTime<Utc>,
    #[serde(skip)]
    pub end_time: DateTime<Utc>,
    #[serde(default)]
    pub length: String,
}



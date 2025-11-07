use std::fmt;
use std::fmt::Formatter;
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
    pub data: f64
}


#[derive(Deserialize)]
pub struct ConsumptionValues {
    #[serde(rename(deserialize = "valid_time"))]
    pub date_time: DateTime<Local>,
    pub data: f64
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
    pub production: Vec<ProductionValues>,
    pub consumption: Vec<ConsumptionValues>,
    pub tariffs: Vec<TariffValues>,
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
    pub start_time: DateTime<Local>,
    pub end_time: DateTime<Local>,
    pub cost: f64,
    pub true_soc_in: Option<usize>,
    pub soc_in: usize,
    pub soc_out: usize,
    pub status: Status,
}

#[derive(Serialize)]
pub struct Block {
    pub block_type: BlockType,
    #[serde(skip_serializing)]
    pub start_time: DateTime<Local>,
    pub cost: String,
    pub true_soc_in: Option<usize>,
    pub soc_in: usize,
    pub soc_out: usize,
    pub status: String,
    #[serde(default)]
    pub start: String,
    #[serde(default)]
    pub length: String,
}



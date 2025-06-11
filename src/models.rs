use std::collections::{HashMap, VecDeque};
use chrono::{DateTime, DurationRound, Local, RoundingError, TimeDelta, Utc};
use serde::Serialize;
use crate::manager_mygrid::models::Block;
use crate::serialize_timestamp;
use crate::traits::MyGrid;

#[derive(Serialize)]
pub struct DataPoint<T> {
    pub x: String,
    pub y: T,
}

#[derive(Serialize, Clone)]
pub struct DataItem<T> {
    #[serde(with = "serialize_timestamp")]
    pub x: DateTime<Local>,
    pub y: T,
}

#[derive(Serialize, Clone)]
pub struct Series<'a, T> {
    pub name: String,
    #[serde(rename(serialize = "type"))]
    pub chart_type: String,
    pub data: &'a Vec<T>,
}

pub struct WeatherData {
    pub temp_history: Vec<DataItem<f64>>,
    pub temp_current: f64,
    pub last_end_time: DateTime<Utc>,
}

pub struct HistoryData {
    pub soc_history: Vec<DataItem<u8>>,
    pub prod_history: Vec<DataItem<f64>>,
    pub load_history: Vec<DataItem<f64>>,
    pub last_end_time: DateTime<Utc>
}

pub struct RealTimeData {
    pub soc: u8,
    pub prod: f64,
    pub load: f64,
    pub prod_data: VecDeque<f64>,
    pub load_data: VecDeque<f64>,
    pub timestamp: i64,
}
pub struct MygridData {
    pub forecast_temp: Vec<DataItem<f64>>,
    pub forecast_cloud: Vec<DataItem<f64>>,
    pub prod: Vec<DataItem<f64>>,
    pub load: Vec<DataItem<f64>>,
    pub tariffs_buy: Vec<DataItem<f64>>,
    pub tariffs_sell: Vec<DataItem<f64>>,
    pub policy_tariffs: HashMap<DateTime<Local>, f64>,
}

pub struct PolicyData<'a> {
    pub schedule: &'a Vec<Block>, 
    pub prod: f64, 
    pub load: f64, 
    pub soc: u8, 
    pub policy_tariffs: &'a HashMap<DateTime<Local>, f64>, 
    pub date_time: DateTime<Local>,
}


impl MyGrid for DataItem<f64> {
    type Item = DataItem<f64>;

    fn is_within(&self, start: DateTime<Local>, end: DateTime<Local>) -> bool {
        self.x >= start && self.x < end
    }

    fn date_time_hour(&self) -> Result<DateTime<Local>, RoundingError> {
        self.x.duration_trunc(TimeDelta::hours(1))
    }

    fn create_new(&self, date_time: DateTime<Local>) -> Self::Item {
        Self::Item { x: date_time, y: self.y }
    }
}


use std::collections::{HashMap, VecDeque};
use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Serialize, PartialEq, Eq, Clone)]
pub enum TariffColor {
    Green,
    Yellow,
    Red,
}

#[derive(Serialize)]
pub struct DataPoint<T> {
    pub x: String,
    pub y: T,
}

#[derive(Serialize, Clone)]
pub struct DataItem<T> {
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub x: DateTime<Utc>,
    pub y: T,
}

pub struct TemperatureData<T> {
    pub history: Vec<DataItem<T>>,
    pub current_temp: Option<T>,
    pub perceived_temp: Option<T>,
}

#[derive(Serialize, Clone)]
pub struct Series<'a, T> {
    pub name: String,
    #[serde(rename(serialize = "type"))]
    pub chart_type: String,
    pub data: &'a Vec<T>,
}

pub struct TwoDayMinMax {
    pub yesterday_min: f64,
    pub yesterday_max: f64,
    pub today_min: f64,
    pub today_max: f64,
}

pub struct WeatherData {
    pub temp_history: Vec<DataItem<f64>>,
    pub forecast_temp: Vec<DataItem<f64>>,
    pub forecast_symbol: Vec<DataItem<u8>>,
    pub min_max: TwoDayMinMax,
    pub temp_current: f64,
    pub temp_perceived: f64,
    pub last_end_time: DateTime<Utc>,
}

pub struct ForecastData {
    pub forecast_temp: Vec<DataItem<f64>>,
    pub symbol_code: Vec<DataItem<u8>>,
}

pub struct HistoryData {
    pub soc_history: Vec<DataItem<u8>>,
    pub prod_history: Vec<DataItem<f64>>,
    pub load_history: Vec<DataItem<f64>>,
    pub last_end_time: DateTime<Utc>
}

pub struct RealTimeData {
    pub soc: u8,
    pub soh: u8,
    pub prod: f64,
    pub load: f64,
    pub prod_data: VecDeque<f64>,
    pub load_data: VecDeque<f64>,
    pub timestamp: i64,
}

#[derive(Clone)]
pub struct TariffFees {
    pub variable_fee: f64,
    pub spot_fee_percentage: f64,
    pub energy_tax: f64,
    pub swedish_power_grid: f64,
    pub balance_responsibility: f64,
    pub electric_certificate: f64,
    pub guarantees_of_origin: f64,
    pub fixed: f64,
}

pub struct MygridData {
    pub base_cost: f64,
    pub schedule_cost: f64,
    pub forecast_temp: Vec<DataItem<f64>>,
    pub forecast_cloud: Vec<DataItem<f64>>,
    pub prod: Vec<DataItem<f64>>,
    pub load: Vec<DataItem<f64>>,
    pub tariffs_buy: Vec<DataItem<f64>>,
    pub tariffs_sell: Vec<DataItem<f64>>,
    pub tariff_fees: TariffFees,
    pub policy_tariffs: HashMap<DateTime<Utc>, f64>,
}

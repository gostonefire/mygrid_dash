pub mod models;

use std::time::Duration;
use chrono::{DateTime, Utc};
use reqwest::Client;
use thiserror::Error;
use crate::manager_inverter::models::{DataRecord, HistoryRecord};

pub struct Inverter {
    client: Client,
    host: String,
}

impl Inverter {
    pub fn new(host: &str) -> Result<Inverter, InverterError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;

        Ok(Self {
            client,
            host: host.to_string(),
        })
    }

    /// Asynchronously retrieves the state of charge (SOC) of the battery.
    ///
    pub async fn get_soc(&self) -> Result<u8, InverterError> {
        Ok(self.get_data("battery_soc").await? as u8)
    }

    /// Asynchronously retrieves the state of health (SOH) of the battery.
    ///
    pub async fn get_soh(&self) -> Result<u8, InverterError> {
        Ok(self.get_data("battery_soh").await? as u8)
    }

    /// Asynchronously retrieves the load power of the household.
    ///
    pub async fn get_load_power(&self) -> Result<f64, InverterError> {
        self.get_data("load_power").await
    }

    /// Asynchronously retrieves the photovoltaic power from the solar panels.
    ///
    pub async fn get_pv_power(&self) -> Result<f64, InverterError> {
        self.get_data("pv1_power").await
    }

    /// Requests data from the inverter.
    ///
    /// # Arguments
    ///
    /// * `register_id` - The ID of the register to fetch data from.
    async fn get_data(&self, register_id: &str) -> Result<f64, InverterError> {
        let url = format!("http://{}/id/{}", self.host, register_id);
        let req = self.client.get(&url).send().await?;
        
        let status = req.status();
        if !status.is_success() {
            return Err(InverterError::InverterError(format!("response with status: {:?}", status)));
        }
        
        let json = req.text().await?;
        let soc: DataRecord<f64> = serde_json::from_str(&json)?;
        
        Ok(soc.data)
    }
    
    /// Requests history data from the inverter.
    /// 
    /// # Arguments
    ///
    /// * `from` - start timestamp for the query
    /// * `to` - end timestamp for the query
    /// * `interval` - interval between samples in minutes (i.e., bucket size)
    pub async fn get_history(&self, from: DateTime<Utc>, to: DateTime<Utc>, interval: i64) -> Result<HistoryRecord, InverterError> {
        let url = format!("http://{}/history?from_ts={}&to_ts={}&interval={}", self.host, from.timestamp(), to.timestamp(), interval);
        
        let from_ts = from.timestamp();
        let to_ts = to.timestamp();
        
        let req = self.client.get(&url)
            .query(&[("from_ts", from_ts), ("to_ts", to_ts), ("interval", interval)])
            .send().await?;
        
        let status = req.status();
        if !status.is_success() {
            return Err(InverterError::InverterError(format!("response with status: {:?}", status)));
        }
        
        let json = req.text().await?;
        let history: HistoryRecord = serde_json::from_str(&json)?;
        
        Ok(history)
    }
}


#[derive(Error, Debug)]
pub enum InverterError {
    #[error("NetworkError: {0}")]
    NetworkError(#[from] reqwest::Error),
    #[error("JsonError: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("InverterError: {0}")]
    InverterError(String),
}
mod models;

use std::time::Duration;
use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use reqwest::Client;
use anyhow::Result;
use thiserror::Error;
use crate::models::{DataItem, TariffFees};
use crate::manager_nordpool::models::Tariffs;

pub struct NordPool {
    client: Client,
    tariff_fees: Option<TariffFees>,
}

impl NordPool {
    pub fn new() -> Result<NordPool, NordPoolError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;

        Ok(Self {
            client,
            tariff_fees: None,
        })
    }

    /// Retrieves day ahead prices from NordPool
    /// It gets the tariffs for the day indicated by date_time (if it can't an error will be returned),
    ///
    /// # Arguments
    ///
    /// * 'day_start' - the start time of the day to retrieve prices for
    /// * 'day_end' - the end time of the day to retrieve prices for (non-inclusive)
    /// * 'day_date' - the date to retrieve prices for
    pub async fn get_tariffs(&self, day_start: DateTime<Utc>, day_end: DateTime<Utc>, day_date: NaiveDate) -> Result<Option<Vec<DataItem<f64>>>, NordPoolError> {
        if self.tariff_fees.is_some() {
            let day_date_utc = TimeZone::from_utc_datetime(&Utc, &day_date.and_hms_opt(0,0,0).unwrap());
            match self.get_day_tariffs(day_start, day_end, day_date_utc).await {
                Ok(result) => Ok(Some(result)),
                Err(e) if matches!(e, NordPoolError::NoContentError) => Ok(None),
                Err(e) => Err(e),
            }
        } else {
            Ok(None)
        }
    }

    /// Sets tariff fees
    /// 
    /// # Arguments
    /// 
    /// * 'tariff_fees' - the fees
    pub fn set_tariff_fees(&mut self, mut tariff_fees: TariffFees) {
        tariff_fees.spot_fee_percentage /= 100.0;
        self.tariff_fees = Some(tariff_fees);
    }
    
    /// Retrieves day ahead prices from NordPool
    ///
    /// # Arguments
    ///
    /// * 'day_start' - the start time of the day to retrieve prices for
    /// * 'day_end' - the end time of the day to retrieve prices for (non-inclusive)
    /// * 'day_date' - the date to retrieve prices for
    async fn get_day_tariffs(&self, day_start: DateTime<Utc>, day_end: DateTime<Utc>, day_date: DateTime<Utc>) -> Result<Vec<DataItem<f64>>, NordPoolError> {
        // https://dataportal-api.nordpoolgroup.com/api/DayAheadPrices?date=2025-10-22&market=DayAhead&deliveryArea=SE4&currency=SEK
        let url = "https://dataportal-api.nordpoolgroup.com/api/DayAheadPrices";
        let date = format!("{}", day_date.format("%Y-%m-%d"));
        let query = vec![
            ("date", date.as_str()),
            ("market", "DayAhead"),
            ("deliveryArea", "SE4"),
            ("currency", "SEK"),
        ];

        let req = self.client.get(url).query(&query).send().await?;

        let status = req.status();
        if status.as_u16() == 204 {
            return Err(NordPoolError::NoContentError)?;
        }

        let json = req.text().await?;

        let tariffs: Tariffs = serde_json::from_str(&json)?;
        self.tariffs_to_vec(&tariffs, day_start, day_end)
    }

    /// Transforms the Tariffs struct to a plain vector of prices
    ///
    /// # Arguments
    ///
    /// * 'tariffs' - the struct containing prices
    /// * 'day_start' - start of day to transform tariffs for
    /// * 'day_end' - end of day to transform tariffs for (non-inclusive)
    fn tariffs_to_vec(&self, tariffs: &Tariffs, day_start: DateTime<Utc>, day_end: DateTime<Utc>) -> Result<Vec<DataItem<f64>>, NordPoolError> {
        let entries = tariffs.multi_area_entries.len();
        if entries < 92 {
            return Err(NordPoolError::ContentLengthError)?
        }
        let day_avg = tariffs.multi_area_entries.iter().map(|t| t.entry_per_area.se4).sum::<f64>() / entries as f64 / 1000.0;

        let mut result: Vec<DataItem<f64>> = Vec::new();
        tariffs.multi_area_entries.iter().filter(|t| t.delivery_start >= day_start && t.delivery_start < day_end).for_each(
            |t| {
                result.push(self.add_vat_markup(day_avg, t.entry_per_area.se4, t.delivery_start));
            });

        Ok(result)
    }

    /// Adds VAT and other markups such as energy taxes etc.
    ///
    /// # Arguments
    ///
    /// * 'day_avg' - average tariff for the day as from NordPool in SEK/MWh
    /// * 'tariff' - spot fee as from NordPool in SEK/MWh
    /// * 'delivery_start' - start time for the spot
    fn add_vat_markup(&self, day_avg: f64, tariff: f64, delivery_start: DateTime<Utc>) -> DataItem<f64> {
        let fees = self.tariff_fees.as_ref().unwrap();

        let price = tariff / 1000.0; // SEK per MWh to per kWh
        let grid_fees = (fees.variable_fee + fees.energy_tax) / 100.0 + fees.spot_fee_percentage * day_avg;
        let trade_fees = (fees.swedish_power_grid + fees.balance_responsibility + fees.electric_certificate +
            fees.guarantees_of_origin + fees.fixed) / 100.0 + price;

        let buy = (grid_fees + trade_fees) / 0.8;

        DataItem { x: delivery_start, y: round_to_two_decimals(buy) }
    }
}


/// Rounds values to two decimals
///
/// # Arguments
///
/// * 'price' - the price to round to two decimals
fn round_to_two_decimals(price: f64) -> f64 {
    (price * 100f64).round() / 100f64
}

#[derive(Error, Debug)]
pub enum NordPoolError {
    #[error("DocumentError: {0}")]
    DocumentError(#[from] serde_json::Error),
    #[error("NetworkError: {0}")]
    NetworkError(#[from] reqwest::Error),
    #[error("NoContentError")]
    NoContentError,
    #[error("ContentLengthError")]
    ContentLengthError,
}
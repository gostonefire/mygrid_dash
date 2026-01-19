use std::ops::Add;
use chrono::{DateTime, Local, TimeDelta, Utc};
use thiserror::Error;
use crate::manager_mygrid::models::{BaseData, Block, SourceBlock};
use crate::models::{DataItem, MygridData, TariffFees};

pub mod models;

/// Size of the smallest block possible in minutes
const BLOCK_UNIT_SIZE: i64 = 15;

/// Reads current schedule from mygrid and returns the block(s)
/// 
/// # Arguments
/// 
/// * 'schedule_path' - full path to the schedule from mygrid
pub async fn get_schedule(schedule_path: &str) -> Result<Vec<Block>, MyGridError> {
    let json = tokio::fs::read_to_string(schedule_path).await?;
    
    let source_blocks: Vec<SourceBlock> = serde_json::from_str(&json)?;

    let blocks: Vec<Block> = source_blocks.iter().map(|b| transform_source_block(b)).collect();

    Ok(blocks)
}

/// Reads base data from mygrid and returns a `BaseData` struct
/// 
/// # Arguments
/// 
/// * 'base_data_path' - full path to the base data file from mygrid
/// * 'utc_now' - date time to check a valid base data file for
/// * 'day_start' - start of day to filter for
/// * 'day_end' - end of day to filter for (non-inclusive)
pub async fn get_base_data(base_data_path: &str, utc_now: DateTime<Utc>, day_start: DateTime<Utc>, day_end: DateTime<Utc>) -> Result<MygridData, MyGridError> {

    let mut mygrid = MygridData {
        base_cost: 0.0,
        schedule_cost: 0.0,
        forecast_temp: Vec::new(),
        forecast_cloud: Vec::new(),
        prod: Vec::new(),
        load: Vec::new(),
        tariff_fees: TariffFees {
            variable_fee: 0.0,
            spot_fee_percentage: 0.0,
            energy_tax: 0.0,
            swedish_power_grid: 0.0,
            balance_responsibility: 0.0,
            electric_certificate: 0.0,
            guarantees_of_origin: 0.0,
            fixed: 0.0,
        },
    };

    let json = get_latest_base_data_content(base_data_path, utc_now).await?;

    if let Some(json) = json {
        let base_data: BaseData = serde_json::from_str(&json)?;
        mygrid.base_cost = base_data.base_cost;
        mygrid.schedule_cost = base_data.schedule_cost;
        mygrid.tariff_fees.variable_fee = base_data.tariff_fees.variable_fee;
        mygrid.tariff_fees.spot_fee_percentage = base_data.tariff_fees.spot_fee_percentage;
        mygrid.tariff_fees.energy_tax = base_data.tariff_fees.energy_tax;
        mygrid.tariff_fees.swedish_power_grid = base_data.tariff_fees.swedish_power_grid;
        mygrid.tariff_fees.balance_responsibility = base_data.tariff_fees.balance_responsibility;
        mygrid.tariff_fees.electric_certificate = base_data.tariff_fees.electric_certificate;
        mygrid.tariff_fees.guarantees_of_origin = base_data.tariff_fees.guarantees_of_origin;
        mygrid.tariff_fees.fixed = base_data.tariff_fees.fixed;

        base_data.forecast.into_iter().filter(|f| f.date_time >= day_start && f.date_time < day_end).for_each(|f| {
            mygrid.forecast_temp.push(DataItem { x: f.date_time, y: f.temp });
            mygrid.forecast_cloud.push(DataItem { x: f.date_time, y: 1.0 - f.cloud_factor });
        });

        base_data.production.into_iter().filter(|d| d.date_time >= day_start && d.date_time < day_end).for_each(|d| {
            mygrid.prod.push(DataItem { x: d.date_time, y: to_kw(d.data, 1) });
        });

        base_data.consumption.into_iter().filter(|d| d.date_time >= day_start && d.date_time < day_end).for_each(|d| {
            mygrid.load.push(DataItem { x: d.date_time, y: to_kw(d.data, 1) });
        });
    }

    Ok(mygrid)
}

/// Finds and reads the latest base data file that is equal to or older than the target time
///
/// # Arguments
///
/// * 'dir_path' - path to the base data dir
/// * 'target_time' - the time to find the latest base data file for
pub async fn get_latest_base_data_content(dir_path: &str, target_time: DateTime<Utc>) -> Result<Option<String>, MyGridError> {
    let limit_filename = format!("{}_base_data.json", target_time.format("%Y%m%d%H%M"));
    let pattern = format!("{}/*_base_data.json", dir_path);

    let latest_file = glob::glob(&pattern)?
        .filter_map(Result::ok)
        .filter_map(|p| p.file_name()?.to_str().map(|s| s.to_string()))
        .filter(|name| name <= &limit_filename)
        .max();

    if let Some(file_name) = latest_file {
        let full_path = std::path::Path::new(dir_path).join(file_name);
        let content = tokio::fs::read_to_string(full_path).await?;
        Ok(Some(content))
    } else {
        Ok(None)
    }
}


/// Converts and rounds from watts to kWh
/// 
/// # Arguments
/// 
/// * 'w' - input in watts
/// * 'units_per_hour' - i.e. data points per hour
fn to_kw(w: f64, units_per_hour: i64) -> f64 {
    (w * units_per_hour as f64 / 10.0).round() / 100.0
}

/// Transforms the Block as given from MyGrid, i.e. the SourceBlock the dash representation of a Block
///
/// # Arguments
///
/// * 'block' - input block of type SourceBlock
fn transform_source_block(block: &SourceBlock) -> Block {
    let length = block.end_time.add(TimeDelta::minutes(BLOCK_UNIT_SIZE)) - block.start_time;

    Block {
        block_type: block.block_type.clone(),
        cost: format!("{:05.2}", block.cost),
        true_soc_in: block.true_soc_in,
        soc_in: block.soc_in,
        soc_out: block.soc_out,
        status: block.status.to_string(),
        start: block.start_time.with_timezone(&Local).format("%H:%M").to_string(),
        start_time: block.start_time,
        end_time: block.end_time.add(TimeDelta::minutes(BLOCK_UNIT_SIZE)),
        length: format!("{:02}:{:02}", length.num_hours(), length.num_minutes() - length.num_hours() * 60),
    }
}

#[derive(Debug, Error)]
pub enum MyGridError {
    #[error("FileReadError: {0}")]
    FileReadError(#[from] std::io::Error),
    #[error("JsonError: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("ChronoParseError: {0}")]
    ChronoParseError(#[from] chrono::format::ParseError),
    #[error("GlobPatternError: {0}")]
    GlobPatternError(#[from] glob::PatternError),
}

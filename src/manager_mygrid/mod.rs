use std::collections::HashMap;
use std::ops::Add;
use chrono::{DateTime, Local, TimeDelta, Utc};
use crate::manager_mygrid::errors::MyGridError;
use crate::manager_mygrid::models::{BaseData, Block, SourceBlock};
use crate::models::{DataItem, MygridData};

pub mod models;
pub mod errors;

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
/// * 'day_start' - start date and time for the day worth of data to be returned
pub async fn get_base_data(base_data_path: &str, day_start: DateTime<Utc>) -> Result<MygridData, MyGridError> {

    let file_path = format!("{}{}_base_data.json", base_data_path, day_start.format("%Y%m%d%H%M"));

    let mut mygrid = MygridData {
        base_cost: 0.0,
        schedule_cost: 0.0,
        forecast_temp: Vec::new(),
        forecast_cloud: Vec::new(),
        prod: Vec::new(),
        load: Vec::new(),
        tariffs_buy: Vec::new(),
        tariffs_sell: Vec::new(),
        policy_tariffs: HashMap::new(),
    };

    if tokio::fs::try_exists(&file_path).await? {
        let json = tokio::fs::read_to_string(file_path).await?;
        let base_data: BaseData = serde_json::from_str(&json)?;
        mygrid.base_cost = base_data.base_cost;
        mygrid.schedule_cost = base_data.schedule_cost;

        for d in base_data.forecast {
            mygrid.forecast_temp.push(DataItem { x: d.date_time, y: d.temp });
            mygrid.forecast_cloud.push(DataItem { x: d.date_time, y: 1.0 - d.cloud_factor });
        }

        for d in base_data.production {
            mygrid.prod.push(DataItem { x: d.date_time, y: to_kw(d.data, 1) });
        }

        for d in base_data.consumption {
            mygrid.load.push(DataItem { x: d.date_time, y: to_kw(d.data, 1) });
        }

        for d in base_data.tariffs {
            mygrid.tariffs_buy.push(DataItem { x: d.date_time, y: d.buy });
            mygrid.tariffs_sell.push(DataItem { x: d.date_time, y: d.sell });
            mygrid.policy_tariffs.insert(d.date_time, d.buy);
        }
    }

    Ok(mygrid)
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
        length: format!("{:02}:{:02}", length.num_hours(), length.num_minutes() - length.num_hours() * 60),
    }
}
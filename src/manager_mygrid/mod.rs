use std::ops::Add;
use chrono::{DateTime, DurationRound, Local, TimeDelta};
use crate::manager_mygrid::errors::MyGridError;
use crate::manager_mygrid::models::{BaseData, Block};
use crate::models::{DataItem, MygridData};

pub mod models;
pub mod errors;


/// Reads current schedule from mygrid and returns the block(s)
/// 
/// # Arguments
/// 
/// * 'schedule_path' - full path to the schedule from mygrid
pub async fn get_schedule(schedule_path: &str) -> Result<Vec<Block>, MyGridError> {
    let json = tokio::fs::read_to_string(schedule_path).await?;
    
    let blocks: Vec<Block> = serde_json::from_str(&json)?;
    Ok(blocks)
}

/// Reads base data from mygrid and returns a `BaseData` struct
/// 
/// # Arguments
/// 
/// * 'base_data_path' - full path to the base data file from mygrid
pub async fn get_base_data(base_data_path: &str) -> Result<MygridData, MyGridError> {
    let json = tokio::fs::read_to_string(base_data_path).await?;
    
    let base_data: BaseData = serde_json::from_str(&json)?;
    Ok(transform_base_data(base_data)?)
}

/// Transforms data from vector of structs with values to vector of values
/// 
/// # Arguments
/// 
/// * 'base_data' - BaseData struct to transform
fn transform_base_data(base_data: BaseData) -> Result<MygridData, MyGridError> {
    let mut mygrid = MygridData {
        date_time: base_data.date_time,
        forecast_temp: Vec::new(),
        forecast_cloud: Vec::new(),
        production: Vec::new(),
        consumption: Vec::new(),
        tariffs_buy: Vec::new(),
        tariffs_sell: Vec::new(),
    };
    
    base_data.forecast.into_iter().for_each(|p| {
        mygrid.forecast_temp.push(DataItem { x: p.date_time, y: p.temp });
        mygrid.forecast_cloud.push(DataItem { x: p.date_time, y: p.cloud_factor });
    });

    base_data.production.into_iter().for_each(|p| {
        mygrid.production.push(DataItem{ x: p.date_time, y: to_kw(p.power) });
    });

    base_data.consumption.into_iter().for_each(|p| {
        mygrid.consumption.push(DataItem{ x: p.date_time, y: to_kw(p.power) });
    });

    base_data.tariffs.into_iter().for_each(|p| {
        mygrid.tariffs_buy.push(DataItem { x: p.date_time, y: p.buy });
        mygrid.tariffs_sell.push(DataItem { x: p.date_time, y: p.sell });
    });

    Ok(mygrid)
}


fn to_kw(w: f64) -> f64 {
    (w / 10.0).round() / 100.0
}

fn get_range(vec_in: &Vec<DateTime<Local>>, from: DateTime<Local>, to: DateTime<Local>) -> (usize, usize) {
    let start = vec_in
        .iter()
        .position(|d| d >= &from);

    let end = vec_in
        .iter()
        .rposition(|d| d < &to)
        .map(|d| d+1)
        .unwrap_or(vec_in.len());

    if let Some(start) = start {
        (start, end)
    } else {
        (0, 0)
    }
}

fn get_date_start_end(vec_in: &Vec<DateTime<Local>>) -> Result<Option<(DateTime<Local>, DateTime<Local>)>, MyGridError> {
    if vec_in.len() != 0 {
        let end = vec_in[0].duration_trunc(TimeDelta::hours(1))?;
        let start = end.duration_trunc(TimeDelta::days(1))?;
        
        if end != start {
            return Ok(Some((start, end.add(TimeDelta::hours(-1)))));
        }
    }
    
    Ok(None)
}

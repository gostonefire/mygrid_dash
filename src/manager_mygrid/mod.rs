use std::collections::HashMap;
use std::path::PathBuf;
use chrono::{DateTime, Local, TimeDelta, Utc};
use glob::glob;
use crate::manager_mygrid::errors::MyGridError;
use crate::manager_mygrid::models::{BaseData, Block};
use crate::models::{DataItem, MyGrid, MygridData};

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
/// * 'from' - start date for the data to be returned
/// * 'to' - non-inclusive end date for the data to be returned 
pub async fn get_base_data(base_data_path: &str, from: DateTime<Local>, to: DateTime<Local>) -> Result<MygridData, MyGridError> {
    let mut files: Vec<PathBuf> = Vec::new();

    let pattern = format!("{}*_base_data.json", base_data_path);
    for entry in glob(&pattern)? {
        if let Ok(path) = entry {
            files.push(path);            
        }
    }
    files.sort();

    let mut forecast_temp: HashMap<DateTime<Utc>, DataItem<f64>> = HashMap::new();
    let mut forecast_cloud: HashMap<DateTime<Utc>, DataItem<f64>> = HashMap::new();
    let mut production: HashMap<DateTime<Utc>, DataItem<f64>> = HashMap::new();
    let mut consumption: HashMap<DateTime<Utc>, DataItem<f64>> = HashMap::new();
    let mut tariffs_buy: HashMap<DateTime<Utc>, DataItem<f64>> = HashMap::new();
    let mut tariffs_sell: HashMap<DateTime<Utc>, DataItem<f64>> = HashMap::new();
    let mut tariffs_policy: HashMap<DateTime<Local>, f64> = HashMap::new();

    for file in files {
        let json = tokio::fs::read_to_string(file).await?;
        let base_data: BaseData = serde_json::from_str(&json)?;
        
        for d in base_data.forecast {
            let datetime = d.date_time.with_timezone(&Utc);
            forecast_temp.insert(datetime, DataItem { x: d.date_time, y: d.temp });
            forecast_cloud.insert(datetime, DataItem { x: d.date_time, y: 1.0 - d.cloud_factor });
        }

        for d in base_data.production {
            let datetime = d.date_time.with_timezone(&Utc);
            production.insert(datetime, DataItem { x: d.date_time, y: to_kw(d.power) });
        }

        for d in base_data.consumption {
            let datetime = d.date_time.with_timezone(&Utc);
            consumption.insert(datetime, DataItem { x: d.date_time, y: to_kw(d.power) });
        }

        for d in base_data.tariffs {
            let datetime = d.date_time.with_timezone(&Utc);
            tariffs_buy.insert(datetime, DataItem { x: d.date_time, y: d.buy });
            tariffs_sell.insert(datetime, DataItem { x: d.date_time, y: d.sell });
            tariffs_policy.insert(d.date_time, d.buy);
        }
    }

    let mut mygrid = MygridData {
        forecast_temp: Vec::new(),
        forecast_cloud: Vec::new(),
        prod: Vec::new(),
        load: Vec::new(),
        tariffs_buy: Vec::new(),
        tariffs_sell: Vec::new(),
        policy_tariffs: tariffs_policy,
    };

    let model = DataItem { x: Default::default(), y: 0.0 };
    move_filter_pad(forecast_temp, &mut mygrid.forecast_temp, &model, from, to)?;
    move_filter_pad(forecast_cloud, &mut mygrid.forecast_cloud, &model, from, to)?;
    move_filter_pad(production, &mut mygrid.prod, &model, from, to)?;
    move_filter_pad(consumption, &mut mygrid.load, &model, from, to)?;
    move_filter_pad(tariffs_buy, &mut mygrid.tariffs_buy, &model, from, to)?;
    move_filter_pad(tariffs_sell, &mut mygrid.tariffs_sell, &model, from, to)?;

    Ok(mygrid)
}

/// Moves data between source and target if the data is within the given range, also left pads 
/// if necessary
/// 
/// # Arguments
/// 
/// * 'source' - data source
/// * 'target' - data target
/// * 'model' - the model struct which has the value fields according to what the padded dates should be set to
/// * 'from' - the start date to compare with and the start to pad from
/// * 'to' - the non-inclusive end date to compare with and the stop to pad to
fn move_filter_pad<T: MyGrid<Item = T>>(mut source: HashMap<DateTime<Utc>, T>, target: &mut Vec<T>, model: &T, from: DateTime<Local>, to: DateTime<Local>) -> Result<(), MyGridError> {
    let mut keys = source.keys().map(|d| *d).collect::<Vec<DateTime<Utc>>>();
    keys.sort();
    keys
        .into_iter()
        .map(|k| source.remove(&k).unwrap())
        .filter(|d| d.is_within(from, to))
        .for_each(|d| target.push(d));

    let mut end = if let Some(t) = target.get(0) {
        t.date_time_hour()?
    } else {
        to
    };

    while from < end {
        end += TimeDelta::hours(-1);
        target.insert(0, model.create_new(end));
    }
    
    Ok(())
}


/// Converts and rounds from watts to kilowatts
/// 
/// # Arguments
/// 
/// * 'w' - input in watts
fn to_kw(w: f64) -> f64 {
    (w / 10.0).round() / 100.0
}

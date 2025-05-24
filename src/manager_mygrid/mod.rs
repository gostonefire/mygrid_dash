use std::ops::Add;
use chrono::{DateTime, DurationRound, Local, TimeDelta};
use crate::manager_mygrid::errors::MyGridError;
use crate::manager_mygrid::models::{BaseData, Block, Consumption, Forecast, Mygrid, MygridData, Production, Tariffs};

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
        forecast: Forecast { date_time: Vec::new(), temp: Vec::new(), cloud_factor: Vec::new() },
        production: Production { date_time: Vec::new(), power: Vec::new() },
        consumption: Consumption { date_time: Vec::new(), power: Vec::new() },
        tariffs: Tariffs { date_time: Vec::new(), buy: Vec::new(), sell: Vec::new() },
    };
    
    base_data.forecast.into_iter().for_each(|p| {
        mygrid.forecast.date_time.push(p.date_time);
        mygrid.forecast.temp.push(p.temp);
        mygrid.forecast.cloud_factor.push(p.cloud_factor);
    });

    base_data.production.into_iter().for_each(|p| {
        mygrid.production.date_time.push(p.date_time);
        mygrid.production.power.push(p.power);
    });

    base_data.consumption.into_iter().for_each(|p| {
        mygrid.consumption.date_time.push(p.date_time);
        mygrid.consumption.power.push(p.power);
    });

    base_data.tariffs.into_iter().for_each(|p| {
        mygrid.tariffs.date_time.push(p.date_time);
        mygrid.tariffs.buy.push(p.buy);
        mygrid.tariffs.sell.push(p.sell);
    });

    Ok(mygrid)
}

impl Mygrid for Forecast {
    type Item = Forecast;

    fn keep(&self, from: DateTime<Local>, to: DateTime<Local>) -> Self {
        let (start, end) = get_range(&self.date_time, from, to);

        Forecast {
            date_time: self.date_time[start..end].to_vec(),
            temp: self.temp[start..end].to_vec(),
            cloud_factor: self.cloud_factor[start..end].to_vec(),
        }
    }
    
    fn append_tail(mut self, other: &mut Forecast) -> Self {
        self.date_time.append(&mut other.date_time);
        self.temp.append(&mut other.temp);
        self.cloud_factor.append(&mut other.cloud_factor);
        
        self
    }
    
    fn pad(mut self) -> Result<Self, MyGridError> {
        if let Some((start, mut end)) = get_date_start_end(&self.date_time)? {
            while start <= end {
                self.date_time.insert(0, end);
                self.temp.insert(0, 0.0);
                self.cloud_factor.insert(0, 0.0);
                end = end.add(TimeDelta::hours(-1));
            }
        }

        Ok(self)
    }
}

impl Mygrid for Production {
    type Item = Production;
    
    fn keep(&self, from: DateTime<Local>, to: DateTime<Local>) -> Self {
        let (start, end) = get_range(&self.date_time, from, to);

        Production {
            date_time: self.date_time[start..end].to_vec(),
            power: self.power[start..end].to_vec(),
        }
    }
    fn append_tail(mut self, other: &mut Production) -> Self {
        self.date_time.append(&mut other.date_time);
        self.power.append(&mut other.power);
        
        self
    }
    fn pad(mut self) -> Result<Self, MyGridError> {
        if let Some((start, mut end)) = get_date_start_end(&self.date_time)? {
            while start <= end {
                self.date_time.insert(0, end);
                self.power.insert(0, 0.0);
                end = end.add(TimeDelta::hours(-1));
            }
        }

        Ok(self)
    }
}

impl Mygrid for Consumption {
    type Item = Consumption;

    fn keep(&self, from: DateTime<Local>, to: DateTime<Local>) -> Self {
        let (start, end) = get_range(&self.date_time, from, to);

        Consumption {
            date_time: self.date_time[start..end].to_vec(),
            power: self.power[start..end].to_vec(),
        }
    }
    fn append_tail(mut self, other: &mut Consumption) -> Self{
        self.date_time.append(&mut other.date_time);
        self.power.append(&mut other.power);
        
        self
    }
    fn pad(mut self) -> Result<Self, MyGridError> {
        if let Some((start, mut end)) = get_date_start_end(&self.date_time)? {
            while start <= end {
                self.date_time.insert(0, end);
                self.power.insert(0, 0.0);
                end = end.add(TimeDelta::hours(-1));
            }
        }

        Ok(self)
    }
}

impl Mygrid for Tariffs {
    type Item = Tariffs;

    fn keep(&self, from: DateTime<Local>, to: DateTime<Local>) -> Self {
        let (start, end) = get_range(&self.date_time, from, to);

        Tariffs {
            date_time: self.date_time[start..end].to_vec(),
            buy: self.buy[start..end].to_vec(),
            sell: self.sell[start..end].to_vec(),
        }
    }

    fn append_tail(mut self, other: &mut Tariffs) -> Self {
        self.date_time.append(&mut other.date_time);
        self.buy.append(&mut other.buy);
        self.sell.append(&mut other.sell);

        self
    }

    fn pad(mut self) -> Result<Self, MyGridError> {
        if let Some((start, mut end)) = get_date_start_end(&self.date_time)? {
            while start <= end {
                self.date_time.insert(0, end);
                self.buy.insert(0, 0.0);
                self.sell.insert(0, 0.0);
                end = end.add(TimeDelta::hours(-1));
            }
        }

        Ok(self)
    }
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
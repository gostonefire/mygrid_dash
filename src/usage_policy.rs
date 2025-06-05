use std::collections::HashMap;
use std::ops::Add;
use chrono::{DateTime, Datelike, Local, TimeDelta};
use log::error;
use crate::manager_mygrid::models::{Block, BlockType};
use crate::models::PolicyData;

#[derive(PartialEq, Eq)]
enum TariffColor {
    Green,
    Yellow,
    Red,
}

/// Evaluates and returns usage policy between 1 and 10 where 1 indicates to not use more 
/// than absolutely necessary
/// 
/// # Arguments
/// 
/// * 'data' - a struct containing all necessary data to evaluate policy
pub fn get_policy(data: PolicyData) -> u8 {
    if data.date_time.month() > 3 || data.date_time.month() < 10 {
        summer_policy(data)
    } else {
        winter_policy(data)
    }
}

/// Policy to be used during April to September where solar production is expected 
/// to be reasonably high
/// 
/// # Arguments
///
/// * 'data' - a struct containing all necessary data to evaluate policy
fn summer_policy(data: PolicyData) -> u8 {
    let consume_date_time = net_consume_at(&data.schedule);
    let color_consume_future = tariff_color_future(consume_date_time, &data.policy_tariffs);
    let color_near_future = tariff_color_future(Some(data.date_time), &data.policy_tariffs);
    let color_now = tariff_color_now(data.date_time, &data.policy_tariffs);
    let is_consuming = data.prod < data.load;
    
    if !is_consuming && data.soc >= 97 {
        10
    } else if data.prod - data.load > 3.0 && data.soc > 20 {
        9
    } else if !is_consuming && data.soc >= 50 {
        8
    } else if is_consuming && data.soc < 50 && consume_date_time.is_none() {
        7
    } else if is_consuming && data.soc < 50 && color_near_future == TariffColor::Green {
        6
    } else if is_consuming && data.soc < 20 && color_near_future == TariffColor::Green {
        5
    } else if is_consuming && data.soc < 50 && consume_date_time.is_some() && color_consume_future == TariffColor::Yellow {
        4
    } else if is_consuming && data.soc < 50 && consume_date_time.is_some() && color_consume_future == TariffColor::Red {
        3
    } else if is_consuming && data.soc <= 10 && color_now == TariffColor::Yellow {
        2
    }  else if is_consuming && data.soc <= 10 && color_now == TariffColor::Red {
        1
    } else {
        error!("summer policy fell through rules");
        1
    }
}

/// Policy to be used during October to March where solar production is expected 
/// to be slim to none
/// 
/// # Arguments
///
/// * 'data' - a struct containing all necessary data to evaluate policy
fn winter_policy(data: PolicyData) -> u8 {
    let color_near_future = tariff_color_future(Some(data.date_time), &data.policy_tariffs);
    let color_now = tariff_color_now(data.date_time, &data.policy_tariffs);
    let is_consuming = data.prod < data.load;

    if !is_consuming && data.soc >= 50 {
        10
    } else if data.soc >= 50 && color_near_future == TariffColor::Green {
        9
    } else if data.soc < 50 && color_near_future == TariffColor::Green {
        8
    } else if data.soc < 20 && color_near_future == TariffColor::Green {
        7
    } else if data.soc >= 50 && color_near_future == TariffColor::Yellow {
        6
    } else if data.soc < 50 && color_near_future == TariffColor::Yellow {
        5
    } else if data.soc >= 50 && color_near_future == TariffColor::Red {
        4
    } else if data.soc < 50 && color_near_future == TariffColor::Red {
        3
    } else if data.soc <= 10 && color_now == TariffColor::Yellow {
        2
    }  else if data.soc <= 10 && color_now == TariffColor::Red {
        1
    } else {
        error!("summer policy fell through rules");
        1
    }
}

/// Returns a datetime where it is estimated that usage may be needed from grid, if no 
/// such date can be found a None is returned instead
///
/// # Arguments
/// 
/// * 'data' - a vector of schedule blocks
fn net_consume_at(data: &Vec<Block>) -> Option<DateTime<Local>> {
    data.iter().rev().filter(|b| b.block_type != BlockType::Use).last().map(|b| b.start_time)
}

/// Returns the tariff color in the future starting from the given datetime and 4 hours further
/// If no date is given the color is defaulted to Green.
/// 
/// # Arguments
/// 
/// * 'date_time' - a datetime (hour) to start evaluating from
/// * 'tariffs' - hourly buy tariffs
fn tariff_color_future(date_time: Option<DateTime<Local>>, tariffs: &HashMap<DateTime<Local>, f64>) -> TariffColor {
    if let Some(date_time) = date_time {
        let mut max_cost: f64 = 0.0;

        for i in 0..4i64 {
            if let Some(&cost) = tariffs.get(&date_time.add(TimeDelta::hours(i))) {
                max_cost = max_cost.max(cost);
            }
        }
        cost_to_color(Some(max_cost))
    } else {
        TariffColor::Green
    }
}

/// Returns the tariff color for the given datetime
///
/// # Arguments
///
/// * 'date_time' - a datetime (hour) to evaluate
/// * 'tariffs' - hourly buy tariffs
fn tariff_color_now(date_time: DateTime<Local>, tariffs: &HashMap<DateTime<Local>, f64>) -> TariffColor {
    cost_to_color(tariffs.get(&date_time).map(|&c| c))
}

/// Translates a cost to a color
/// If no cost is given the color defaults from Green
/// 
/// # Arguments
/// 
/// * 'cost' - cost to translate
fn cost_to_color(cost: Option<f64>) -> TariffColor {
    if let Some(cost) = cost {
        if cost > 4.0 {
            TariffColor::Red
        } else if cost > 2.0 {
            TariffColor::Yellow
        } else {
            TariffColor::Green
        }
    } else {
        TariffColor::Green
    }
}
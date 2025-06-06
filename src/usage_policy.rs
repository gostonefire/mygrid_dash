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
    let soc_full = data.soc > 96;
    let soc_hi = data.soc > 50;
    let soc_med = data.soc > 20;
    let soc_low = data.soc > 10;
    let soc_empty = true;
    
    let far_color = tariff_color_future(net_consume_at(&data.schedule), &data.policy_tariffs);
    let near_color = tariff_color_future(Some(data.date_time), &data.policy_tariffs).unwrap_or(TariffColor::Green);
    let now_color = tariff_color_now(data.date_time, &data.policy_tariffs);
    let is_prod = data.prod > data.load;
    
    if soc_full && far_color.is_none() && is_prod {
        10
    } else if soc_full && far_color.is_none() {
        9
    } else if soc_hi && far_color.is_none() {
        8
    } else if soc_hi && is_prod {
        7
    } else if soc_hi && far_color.as_ref().is_some_and(|c| *c == TariffColor::Green) {
        6
    } else if soc_hi && far_color.as_ref().is_some_and(|c| *c == TariffColor::Yellow) {
        5
    } else if soc_hi && far_color.as_ref().is_some_and(|c| *c == TariffColor::Red) {
        4
    } else if soc_med && far_color.is_none() {
        7
    } else if soc_med && far_color.as_ref().is_some_and(|c| *c == TariffColor::Green) {
        6
    }  else if soc_med && far_color.as_ref().is_some_and(|c| *c == TariffColor::Yellow) {
        5
    } else if soc_med && far_color.as_ref().is_some_and(|c| *c == TariffColor::Red) {
        4
    } else if soc_low && near_color == TariffColor::Green {
        4
    } else if soc_low && near_color == TariffColor::Yellow {
        3
    } else if soc_low && near_color == TariffColor::Red {
        2
    } else if soc_empty && now_color == TariffColor::Green {
        3
    } else if soc_empty && now_color == TariffColor::Yellow {
        2
    } else if soc_empty && now_color == TariffColor::Red {
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
    let soc_hi = data.soc > 50;
    let soc_med = data.soc > 20;
    let soc_low = data.soc > 10;
    let soc_empty = true;

    let near_color = tariff_color_future(Some(data.date_time), &data.policy_tariffs).unwrap_or(TariffColor::Green);
    let now_color = tariff_color_now(data.date_time, &data.policy_tariffs);
    let is_prod = data.prod > data.load;

    if soc_hi && is_prod && near_color == TariffColor::Green {
        10
    } else if soc_hi && near_color == TariffColor::Green {
        9
    } else if soc_hi && near_color == TariffColor::Yellow {
        6
    } else if soc_hi && near_color == TariffColor::Red {
        4
    } else if soc_med && near_color == TariffColor::Green {
        8
    } else if soc_med && near_color == TariffColor::Yellow {
        5
    } else if soc_med && near_color == TariffColor::Red {
        3
    } else if soc_low && near_color == TariffColor::Green {
        7
    } else if soc_low && near_color == TariffColor::Yellow {
        4
    } else if soc_low && near_color == TariffColor::Red {
        2
    } else if soc_empty && now_color == TariffColor::Green {
        6
    } else if soc_empty && now_color == TariffColor::Yellow {
        3
    } else if soc_empty && now_color == TariffColor::Red {
        1
    } else {
        error!("winter policy fell through rules");
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
/// 
/// # Arguments
/// 
/// * 'date_time' - a datetime (hour) to start evaluating from
/// * 'tariffs' - hourly buy tariffs
fn tariff_color_future(date_time: Option<DateTime<Local>>, tariffs: &HashMap<DateTime<Local>, f64>) -> Option<TariffColor> {
    if let Some(date_time) = date_time {
        let mut max_cost: f64 = 0.0;

        for i in 0..4i64 {
            if let Some(&cost) = tariffs.get(&date_time.add(TimeDelta::hours(i))) {
                max_cost = max_cost.max(cost);
            }
        }
        Some(cost_to_color(Some(max_cost)))
    } else {
        None
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
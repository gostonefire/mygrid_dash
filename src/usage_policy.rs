use std::collections::HashMap;
use std::iter::successors;
use std::ops::Add;
use chrono::{DateTime, TimeDelta, Utc};
use crate::manager_mygrid::models::{Block, BlockType};
use crate::models::TariffColor;


/// Evaluates and returns usage tariff color
///
/// # Arguments
/// 
/// * 'date_time' - current UTC date time
/// * 'soc' - current state of charge
/// * 'schedule' - schedule of the day, used to determine if the battery is discharging or not
/// * 'tariffs' - hourly buy tariffs
pub fn get_policy(date_time: DateTime<Utc>, soc: u8, schedule: &Vec<Block>, tariffs: &HashMap<DateTime<Utc>, f64>) -> TariffColor {
    // Checks if we are currently discharging from the battery, with a margin of 15%.
    // The margin is used so that we are not displaying a policy of e.g. green when there is only a very small
    //  amount of battery charge left, but the battery is still discharging.
    let is_discharging_with_margin = schedule
        .iter()
        .filter(|block| block.start_time <= date_time && block.end_time > date_time)
        .last()
        .map(|b| soc > (b.soc_out + 15) as u8)
        .unwrap_or(false);

    let last_charge_time = schedule
        .iter()
        .filter(|b| b.block_type == BlockType::Charge && date_time > b.end_time)
        .last()
        .map(|b| (b.start_time, b.end_time));

    let charge_price: Option<f64> = last_charge_time.map(|(start, end)| {
        let mut intervals = 0;
        let total_price: f64 = successors(Some(start), |&t| {
            let next = t + TimeDelta::minutes(15);
            (next < end).then_some(next)
        })
            .inspect(|_| intervals += 1)
            .map(|t| tariffs.get(&t).copied().unwrap_or(0.0))
            .sum();

        total_price / intervals as f64
    });

    let now_color = tariff_color_now(date_time, tariffs);

    if is_discharging_with_margin {
        if charge_price.is_some() {
            cost_to_color(charge_price)
        } else {
            match now_color {
                TariffColor::Yellow => TariffColor::Green,
                TariffColor::Red => TariffColor::Yellow,
                _ => now_color,
            }
        }
    } else {
        now_color
    }
}

/// Returns the tariff color for the given datetime.
/// The color reflects the average of the nearest future (approx 4 quarters or 1 hour)
///
/// # Arguments
///
/// * 'date_time' - a datetime (quarter) to evaluate
/// * 'tariffs' - hourly buy tariffs
fn tariff_color_now(date_time: DateTime<Utc>, tariffs: &HashMap<DateTime<Utc>, f64>) -> TariffColor {
    let price_sum = (0..4).fold((0.0f64, 0u8), |acc, i| {
        let dt = date_time.add(TimeDelta::minutes(i as i64 * 15));
        if let Some(&cost) = tariffs.get(&dt) {
            (acc.0 + cost, acc.1 + 1)
        } else {
            acc
        }
    });

    let price_avg = if price_sum.1 != 0 {
        Some(price_sum.0 / price_sum.1 as f64)
    } else {
        None
    };

    cost_to_color(price_avg)
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
use std::collections::HashMap;
use std::ops::Add;
use chrono::{DateTime, TimeDelta, Utc};
use crate::models::TariffColor;


/// Evaluates and returns usage tariff color
///
/// # Arguments
/// 
/// * 'date_time' - current UTC date time
/// * 'soc' - current state of charge
/// * 'is_discharging' - whether the battery is currently discharging
/// * 'charge_price' - the last average charge price, if any.
/// * 'tariffs' - hourly buy tariffs
pub fn get_policy(date_time: DateTime<Utc>, soc: u8, is_discharging: bool, charge_price: Option<f64>, tariffs: &HashMap<DateTime<Utc>, f64>) -> TariffColor {
    let now_color = tariff_color_now(date_time, tariffs);
    let soc_ok = soc > 25;

    if is_discharging && charge_price.is_some() {
        cost_to_color(charge_price)
    } else if now_color == TariffColor::Yellow && soc_ok && is_discharging {
        TariffColor::Green
    } else if now_color == TariffColor::Red && soc_ok && is_discharging {
        TariffColor::Yellow
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
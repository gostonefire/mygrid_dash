use chrono::{DateTime, Local, RoundingError};
use log::error;
use crate::errors::WeatherError;

pub trait MyGrid {
    type Item;

    /// Returns true if the `Item` is within the given open-ended date range
    ///
    /// # Arguments
    ///
    /// * 'start' - start date time
    /// * 'end' - open-ended end time  
    fn is_within(&self, start: DateTime<Local>, end: DateTime<Local>) -> bool;

    /// Returns the `Item` represented date time truncated to hours
    ///
    fn date_time_hour(&self) -> Result<DateTime<Local>, RoundingError>;

    /// Returns a new instance of type `Item` with the given date_time set
    ///
    fn create_new(&self, date_time: DateTime<Local>) -> Self::Item;
}

pub trait Weather {
    
    /// Returns current temperature
    /// 
    fn get_temperature(&self) -> Result<f64, WeatherError>;
    
    /// Returns temperature history
    /// 
    /// # Arguments
    /// 
    /// * 'from' - history start datetime
    /// * 'to' - history end time (non-inclusive)
    fn get_temperature_history(&self, from: DateTime<Local>, to: DateTime<Local>) -> Result<Vec<(DateTime<Local>, f64)>, WeatherError> {
        error!("get_temperature_history({}, {}) not implemented", from, to);
        Err("get_temperature_history not implemented")?
    }
    
    /// Returns current humidity
    fn get_humidity(&self) -> Result<f64, WeatherError> {
        error!("get_humidity not implemented");
        Err("get_humidity not implemented")?
    }

    /// Returns humidity history
    ///
    /// # Arguments
    ///
    /// * 'from' - history start datetime
    /// * 'to' - history end time (non-inclusive)
    fn get_humidity_history(&self, from: DateTime<Local>, to: DateTime<Local>) -> Result<Vec<(DateTime<Local>, f64)>, WeatherError> {
        error!("get_humidity_history({}, {}) not implemented", from, to);
        Err("get_humidity_history not implemented")?
    }

    /// Returns temperature max within a given period
    ///
    /// # Arguments
    ///
    /// * 'from' - period start datetime
    /// * 'to' - period end time (non-inclusive)
    fn get_temperature_max(&self, from: DateTime<Local>, to: DateTime<Local>) -> Result<f64, WeatherError> {
        error!("get_temperature_max({}, {}) not implemented", from, to);
        Err("get_temperature_max not implemented")?
    }

    /// Returns temperature min within a given period
    ///
    /// # Arguments
    ///
    /// * 'from' - period start datetime
    /// * 'to' - period end time (non-inclusive)
    fn get_temperature_min(&self, from: DateTime<Local>, to: DateTime<Local>) -> Result<f64, WeatherError> {
        error!("get_temperature_min({}, {}) not implemented", from, to);
        Err("get_temperature_min not implemented")?
    }

    /// Returns humidity max within a given period
    ///
    /// # Arguments
    ///
    /// * 'from' - period start datetime
    /// * 'to' - period end time (non-inclusive)
    fn get_humidity_max(&self, from: DateTime<Local>, to: DateTime<Local>) -> Result<f64, WeatherError> {
        error!("get_humidity_max({}, {}) not implemented", from, to);
        Err("get_humidity_max not implemented")?
    }

    /// Returns humidity min within a given period
    ///
    /// # Arguments
    ///
    /// * 'from' - period start datetime
    /// * 'to' - period end time (non-inclusive)
    fn get_humidity_min(&self, from: DateTime<Local>, to: DateTime<Local>) -> Result<f64, WeatherError> {
        error!("get_humidity_min({}, {}) not implemented", from, to);
        Err("get_humidity_min not implemented")?
    }
}
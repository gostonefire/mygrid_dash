use chrono::{DateTime, Local, RoundingError};

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


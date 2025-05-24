use chrono::{DateTime, Local};
use serde::{self, Serializer};
use serde::ser::SerializeSeq;

const FORMAT: &'static str = "%H:%M";

/// Serializer for serde with to serialize a `chrono` DateTime vector into an hour:minute format
/// This function is not used directly but rather from struct fields with a serde with attribute 
/// pointing to this module
/// 
/// # Arguments
/// 
/// * 'date_time' - the vector of date time objects
/// * 'serializer' - serializer given from serde
pub fn serialize<S>(
    date_time: &Vec<DateTime<Local>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut state = serializer.serialize_seq(Some(date_time.len()))?;
    
    for element in date_time {
        let s = format!("{}", element.format(FORMAT));
        state.serialize_element(&s)?;
    }
    state.end()
}

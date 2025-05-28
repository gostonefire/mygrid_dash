use chrono::{DateTime, Local};
use serde::{self, Serialize, Serializer};

const FORMAT: &'static str = "%H";

/// Serializer for serde with to serialize a `chrono` DateTime into an ISO 8601 format
/// This function is not used directly but rather from struct fields with a serde with attribute 
/// pointing to this module
///
/// # Arguments
///
/// * 'date_time' - the date time object
/// * 'serializer' - serializer given from serde
pub fn serialize<S>(
    date: &DateTime<Local>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    date.naive_local().and_utc().timestamp_millis().serialize(serializer)
    // let s = format!("{}", date.format(FORMAT));
    // serializer.serialize_str(&s)
}
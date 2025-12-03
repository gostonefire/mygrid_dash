use chrono::{DateTime, Utc};
use serde::{self, Serialize, Serializer};


/// Serializer for serde with to serialize a chrono `DateTime<Local>` into a millisecond timestamp (Utc)
/// This function is not used directly but rather from struct fields with a serde with attribute 
/// pointing to this module
///
/// # Arguments
///
/// * 'date_time' - the date time object
/// * 'serializer' - serializer given from serde
pub fn serialize<S>(
    date: &DateTime<Utc>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    date.timestamp_millis().serialize(serializer)
}
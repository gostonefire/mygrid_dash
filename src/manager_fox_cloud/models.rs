use chrono::{NaiveDate};
use serde::{Deserialize, Deserializer, Serialize};
use serde::de::Error;
use serde_json::Value;

#[derive(Serialize)]
pub struct RequestDeviceHistoryData {
    pub sn: String,
    pub variables: Vec<String>,
    pub begin: i64,
    pub end: i64,
}


#[derive(Serialize, Deserialize)]
pub struct Data {
    pub time: String,
    #[serde(deserialize_with = "deserialize_scientific_notation")]
    pub value: f64,
}

#[derive(Deserialize)]
pub struct DataSet {
    pub data: Vec<Data>,
    pub variable: String,
}

#[derive(Deserialize)]
pub struct DeviceHistoryData {
    #[serde(rename = "datas")]
    pub data_set: Vec<DataSet>,
}

#[derive(Deserialize)]
pub struct DeviceHistoryResult {
    pub result: Vec<DeviceHistoryData>,
}

#[derive(Debug)]
pub struct DeviceHistory {
    pub date: NaiveDate,
    pub time: Vec<String>,
    pub pv_power: Vec<f64>,
    pub ld_power: Vec<f64>,
    pub soc: Vec<u8>,
}

fn deserialize_scientific_notation<'de, D>(deserializer: D) -> Result<f64, D::Error>
where D: Deserializer<'de> {

    let v = Value::deserialize(deserializer)?;
    let x = v.as_f64()
        .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
        .ok_or_else(|| Error::custom("non-f64"))?
        .try_into()
        .map_err(|_| Error::custom("overflow"))?;

    Ok(x)
}


#[derive(Deserialize)]
pub struct SocCurrentResult {
    pub result: Vec<SocCurrentVariables>,
}

#[derive(Deserialize)]
pub struct SocCurrentVariables {
    pub datas: Vec<SocCurrentData>,
    //pub time: String,
    //#[serde(rename = "deviceSN")]
    //pub device_sn: String,
}

#[derive(Deserialize)]
pub struct SocCurrentData {
    //pub unit: String,
    //pub name: String,
    //pub variable: String,
    pub value: f64,
}

#[derive(Serialize)]
pub struct RequestCurrentSoc {
    pub sn: String,
    pub variables: Vec<String>,
}

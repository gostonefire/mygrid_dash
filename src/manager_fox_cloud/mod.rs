pub mod errors;
mod models;

use std::str::FromStr;
use std::time::Duration;
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use md5::{Digest, Md5};
use serde::{Deserialize, Serialize};
use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use crate::initialization::FoxESS;
use crate::manager_fox_cloud::errors::FoxError;
use crate::manager_fox_cloud::models::{DeviceHistory, DeviceHistoryData, DeviceHistoryResult, DeviceRealTime, DeviceRealTimeResult, RequestDeviceHistoryData, RequestDeviceRealTimeData};

const REQUEST_DOMAIN: &str = "https://www.foxesscloud.com";

pub struct Fox {
    api_key: String,
    sn: String,
    client: Client,
}

impl Fox {
    /// Returns a new instance of the Fox struct
    ///
    /// # Arguments
    ///
    /// * 'config' - FoxESS configuration struct
    pub fn new(config: &FoxESS) -> Result<Self, FoxError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;

        Ok(Self { api_key: config.api_key.to_string(), sn: config.inverter_sn.to_string(), client })
    }

    /// Obtain history data from the inverter
    ///
    /// See https://www.foxesscloud.com/public/i18n/en/OpenApiDocument.html#get20device20history20data0a3ca20id3dget20device20history20data4303e203ca3e
    ///
    /// # Arguments
    ///
    /// * 'start' - the start time of the report
    /// * 'end' - the end time of the report
    pub async fn get_device_history_data(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<DeviceHistory, FoxError> {
        let path = "/op/v0/device/history/query";

        let req = RequestDeviceHistoryData {
            sn: self.sn.clone(),
            variables: ["pvPower", "loadsPower", "SoC"]
                .iter().map(|s| s.to_string())
                .collect::<Vec<String>>(),
            begin: start.timestamp_millis(),
            end: end.timestamp_millis(),
        };

        let req_json = serde_json::to_string(&req)?;

        let json = self.post_request(&path, req_json).await?;

        let fox_data: DeviceHistoryResult = serde_json::from_str(&json)?;
        let device_history = transform_history_data(end, fox_data.result)?;

        Ok(device_history)
    }

    /// Obtain real time data from the inverter
    ///
    /// See https://www.foxesscloud.com/public/i18n/en/OpenApiDocument.html#get20device20real-time20data0a3ca20id3dget20device20real-time20data5603e203ca3e
    ///
    pub async fn get_device_real_time_data(&self) -> Result<DeviceRealTime, FoxError> {
        let path = "/op/v1/device/real/query";

        let req = RequestDeviceRealTimeData {
            variables: ["pvPower", "loadsPower", "SoC"]
                .iter().map(|s| s.to_string())
                .collect::<Vec<String>>(),
            sns: vec![self.sn.clone()],
        };

        let req_json = serde_json::to_string(&req)?;

        let json = self.post_request(&path, req_json).await?;

        let fox_data: DeviceRealTimeResult = serde_json::from_str(&json)?;

        let mut device_real_time = DeviceRealTime {
            pv_power: 0.0,
            ld_power: 0.0,
            soc: 0,
        };
        
        for data in fox_data.result[0].datas.iter() {
            match data.variable.as_str() {
                "pvPower" => device_real_time.pv_power = data.value,
                "loadsPower" => device_real_time.ld_power = data.value,
                "SoC" => device_real_time.soc = data.value.round() as u8,
                _ => (),
            }
        }
        
        Ok(device_real_time)
    }

    /// Builds a request and sends it as a POST.
    /// The return is the json representation of the result as specified by
    /// respective FoxESS API
    ///
    /// # Arguments
    ///
    /// * path - the API path excluding the domain
    /// * body - a string containing the payload in json format
    async fn post_request(&self, path: &str, body: String) -> Result<String, FoxError> {
        let url = format!("{}{}", REQUEST_DOMAIN, path);

        //let mut req = self.client.post(url);
        let headers = self.generate_headers(&path, Some(vec!(("Content-Type", "application/json"))));

        let req = self.client.post(url)
            .headers(headers)
            .body(body)
            .send().await?;

        let status = req.status();
        if !status.is_success() {
            return Err(FoxError::FoxCloud(format!("{:?}", status)));
        }

        let json = req.text().await?;
        let fox_res: FoxResponse = serde_json::from_str(&json)?;
        if fox_res.errno != 0 {
            return Err(FoxError::FoxCloud(format!("errno: {}, msg: {}", fox_res.errno, fox_res.msg)));
        }

        Ok(json)
    }

    /// Generates http headers required by Fox Open API, this includes also building a
    /// md5 hashed signature.
    ///
    /// # Arguments
    ///
    /// * 'path' - the path, excluding the domain part, to the FoxESS specific API
    /// * 'extra' - any extra headers to add besides FoxCloud standards
    fn generate_headers(&self, path: &str, extra: Option<Vec<(&str, &str)>>) -> HeaderMap {
        let mut headers = HeaderMap::new();

        let timestamp = Utc::now().timestamp() * 1000;
        let signature = format!("{}\\r\\n{}\\r\\n{}", path, self.api_key, timestamp);

        let mut hasher = Md5::new();
        hasher.update(signature.as_bytes());
        let signature_md5 = hasher.finalize().iter().map(|x| format!("{:02x}", x)).collect::<String>();

        headers.insert("token", HeaderValue::from_str(&self.api_key).unwrap());
        headers.insert("timestamp", HeaderValue::from_str(&timestamp.to_string()).unwrap());
        headers.insert("signature", HeaderValue::from_str(&signature_md5).unwrap());
        headers.insert("lang", HeaderValue::from_str("en").unwrap());

        if let Some(h) = extra {
            h.iter().for_each(|&(k, v)| {
                headers.insert(HeaderName::from_str(k).unwrap(), HeaderValue::from_str(v).unwrap());
            });
        }

        headers
    }
}

/// Transforms device history data to a format easier to save as non-json file
///
/// # Arguments
///
/// * 'last_end_time' - the last given end time when requesting history data
/// * 'input' - the data to transform
fn transform_history_data(last_end_time: DateTime<Utc>, input: Vec<DeviceHistoryData>) -> Result<DeviceHistory, FoxError> {
    let mut time: Vec<DateTime<Utc>> = Vec::new();
    let mut pv_power: Vec<f64> = Vec::new();
    let mut ld_power: Vec<f64> = Vec::new();
    let mut soc: Vec<u8> = Vec::new();

    for set in &input[0].data_set {
        if set.variable == "pvPower" {
            for data in &set.data {
                let date_time = cet_to_utc(&data.time)?;

                time.push(date_time);
                pv_power.push(data.value);
            }
        } else if set.variable == "loadsPower" {
            for data in &set.data {
                ld_power.push(data.value);
            }
        } else if set.variable == "SoC" {
            for data in &set.data {
                soc.push(data.value as u8);
            }
        }
    }

    Ok(DeviceHistory {
        last_end_time,
        time,
        pv_power,
        ld_power,
        soc,
    })
}

/// Converts a date time string in special Fox format to UTC
///
/// # Arguments
///
/// * 'time' - date time string in 2025-12-03 00:08:51 CET+0100 format
fn cet_to_utc(time: &str) -> Result<DateTime<Utc>, FoxError> {
    let dt = DateTime::parse_from_str(&time.replace("+", " +"), "%Y-%m-%d %H:%M:%S %Z %z")?;
    Ok(dt.with_timezone(&Utc))
}

#[derive(Serialize, Deserialize)]
struct FoxResponse {
    errno: u32,
    msg: String,
}



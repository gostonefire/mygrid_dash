pub mod errors;
mod models;

use std::str::FromStr;
use std::time::Duration;
use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, Utc};
use md5::{Digest, Md5};
use serde::{Deserialize, Serialize};
use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use crate::initialization::FoxESS;
use crate::manager_fox_cloud::errors::FoxError;
use crate::manager_fox_cloud::models::{DeviceHistory, DeviceHistoryData, DeviceHistoryResult, RequestDeviceHistoryData};
use crate::manager_fox_cloud::models::{SocCurrentResult, RequestCurrentSoc};

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

    /// Obtain the battery current soc (state of charge)
    ///
    /// See https://www.foxesscloud.com/public/i18n/en/OpenApiDocument.html#get20device20real-time20data0a3ca20id3dget20device20real-time20data4303e203ca3e
    ///
    /// # Arguments
    ///
    pub async fn get_current_soc(&self) -> Result<u8, FoxError> {
        let path = "/op/v0/device/real/query";

        let req = RequestCurrentSoc { sn: self.sn.clone(), variables: vec!["SoC".to_string()] };
        let req_json = serde_json::to_string(&req)?;

        let json = self.post_request(&path, req_json).await?;

        let fox_data: SocCurrentResult = serde_json::from_str(&json)?;

        Ok(fox_data.result[0].datas[0].value.round() as u8)
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
        let device_history = transform_history_data(start.with_timezone(&Local).date_naive(), fox_data.result)?;

        Ok(device_history)
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
/// * 'date' - the date the data is valid for
/// * 'input' - the data to transform
fn transform_history_data(date: NaiveDate, input: Vec<DeviceHistoryData>) -> Result<DeviceHistory, FoxError> {
    let mut time: Vec<String> = Vec::new();
    let mut pv_power: Vec<f64> = Vec::new();
    let mut ld_power: Vec<f64> = Vec::new();
    let mut soc: Vec<u8> = Vec::new();

    for set in &input[0].data_set {
        if set.variable == "pvPower" {
            for data in &set.data {
                let ndt = NaiveDateTime::parse_from_str(&data.time, "%Y-%m-%d %H:%M:%S %Z")?
                    .format("%Y-%m-%d %H:%M").to_string();

                time.push(ndt);
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
        date,
        time,
        pv_power,
        ld_power,
        soc,
    })
}

#[derive(Serialize, Deserialize)]
struct FoxResponse {
    errno: u32,
    msg: String,
}



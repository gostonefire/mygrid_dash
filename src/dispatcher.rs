use std::collections::{HashMap, VecDeque};
use std::ops::Add;
use chrono::{DateTime, Datelike, Duration, DurationRound, Local, NaiveDateTime, TimeDelta, Utc};
use log::{error, info};
use serde::Serialize;
use tokio::select;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use crate::errors::DispatcherError;
use crate::initialization::Config;
use crate::manager_fox_cloud::Fox;
use crate::manager_mygrid::{get_base_data, get_schedule};
use crate::manager_mygrid::models::Block;
use crate::manager_weather::Weather;
use crate::models::{DataItem, DataPoint, HistoryData, MygridData, PolicyData, RealTimeData, Series, MinMax, WeatherData};
use crate::usage_policy::get_policy;

pub enum Cmd {
    SmallDashData,
    FullDashData,
}


/// Sync start point
/// This loop will never end unless some means of stopping it is implemented,but rather
/// report any errors encountered and after some wait try again
///
/// # Arguments
///
/// * 'tx' - mpsc sender to the web server
/// * 'rx' - mpsc receiver from the web server
/// * 'config' - configuration struct
pub async fn run(tx: UnboundedSender<String>,  rx: UnboundedReceiver<Cmd>, config: &Config) {
    let mut disp = match Dispatcher::new(config).await {
        Ok(d) => d,
        Err(e) => {
            error!("while initializing dispatcher: {}", e);
            return;
        }
    };
    let _ = &disp.update_mygrid_data().await;

    match dispatch_loop(tx, rx, &mut disp).await {
        Ok(_) => {
            info!("dispatch loop terminated");
        },
        Err(e) => {
            error!("dispatch loop terminated with error: {}", e);
        }
    }
}

/// Main dispatch loop that regularly read mygrid files and builds up history data
/// while also listening for requests from the web server
///
async fn dispatch_loop(tx: UnboundedSender<String>, mut rx: UnboundedReceiver<Cmd>, disp: &mut Dispatcher) -> Result<(), DispatcherError> {
    let (tx_sleep, mut rx_sleep) = tokio::sync::mpsc::unbounded_channel::<bool>();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(180)).await;
            tx_sleep.send(true).unwrap();
        }
    });
    
    loop {
        select! {
            cmd = rx.recv() => {
                if let Some(cmd) = cmd {
                    let _ = &disp.check_updates(true).await?;

                    let data = disp.execute_cmd(cmd).await?;
                    tx.send(data)?;
                } else {
                    return Err("cmd receiver closed unexpectedly".into());
                }
            },
            wake = rx_sleep.recv() => {
                if wake.is_some() {
                    let _ = &disp.check_updates(false).await?;
                    let _ = &disp.update_mygrid_data().await?;
                } else {
                    return Err("wake receiver closed unexpectedly".into());
                }
            },
            else => return Ok(()),
        }
    }
}


/// Dispatcher struct
///
struct Dispatcher {
    schedule: Vec<Block>,
    mygrid_data: MygridData,
    fox_cloud: Fox,
    weather: Weather,
    schedule_path: String,
    base_data_path: String,
    history_data: HistoryData,
    real_time_data: RealTimeData,
    weather_data: WeatherData,
    usage_policy: u8,
    last_request: i64,
}

impl Dispatcher {
    /// Creates a new `Dispatcher` ready for action
    ///
    /// # Arguments
    ///
    /// * 'config' - configuration struct
    async fn new(config: &Config) -> Result<Self, DispatcherError> {
        let fox_cloud = Fox::new(&config.fox_ess)?;
        let weather = Weather::new(&config.weather.host, &config.weather.sensor)?;
        
        Ok(Self {
            schedule: Vec::new(),
            mygrid_data: MygridData {
                forecast_temp: Vec::new(),
                forecast_cloud: Vec::new(),
                prod: Vec::new(),
                load: Vec::new(),
                tariffs_buy: Vec::new(),
                tariffs_sell: Vec::new(),
                policy_tariffs: HashMap::new(),
            },
            fox_cloud,
            weather,
            schedule_path: config.mygrid.schedule_path.clone(),
            base_data_path: config.mygrid.base_data_path.clone(),
            history_data: HistoryData {
                soc_history: Vec::new(),
                prod_history: Vec::new(),
                load_history: Vec::new(),
                last_end_time: Default::default(),
            },
            real_time_data: RealTimeData {
                soc: 0,
                prod: 0.0,
                load: 0.0,
                prod_data: VecDeque::new(),
                load_data: VecDeque::new(),
                timestamp: 0,
            },
            weather_data: WeatherData {
                temp_history: Vec::new(),
                min_max: MinMax {
                    yesterday_min: 0.0,
                    yesterday_max: 0.0,
                    today_min: 0.0,
                    today_max: 0.0,
                },
                temp_current: 0.0,
                last_end_time: Default::default(),
            },
            usage_policy: 0,
            last_request: 0,
        })
    }

    /// Executes a command and returns the same command but with requested data
    ///
    /// # Arguments
    ///
    /// * 'cmd' - the command to evaluate and execute
    async fn execute_cmd(&mut self, cmd: Cmd) -> Result<String, DispatcherError> {
        let data = match cmd {
            Cmd::SmallDashData       => self.get_small_dash_data()?,
            Cmd::FullDashData        => self.get_full_dash_data()?,
        };

        Ok(data)
    }
    
    /// Returns a json object with all necessary data for the small dash
    /// 
    fn get_small_dash_data(&self) -> Result<String, DispatcherError> {
        #[derive(Serialize)]
        struct SmallDashData<'a> {
            policy: u8,
            temp_current: f64,
            yesterday_min: f64,
            yesterday_max: f64,
            today_min: f64,
            today_max: f64,
            temp_diagram: (Series<'a, DataItem<f64>>, Series<'a, DataItem<f64>>),
            tariffs_buy: Series<'a, DataItem<f64>>,
        }
        
        let reply = SmallDashData {
            policy: self.usage_policy,
            temp_current: self.weather_data.temp_current,
            yesterday_min: self.weather_data.min_max.yesterday_min,
            yesterday_max: self.weather_data.min_max.yesterday_max,
            today_min: self.weather_data.min_max.today_min,
            today_max: self.weather_data.min_max.today_max,
            temp_diagram: (
                Series {
                    name: "Forecast".to_string(),
                    chart_type: String::new(),
                    data: &self.mygrid_data.forecast_temp,
                },
                Series {
                    name: "Actual".to_string(),
                    chart_type: String::new(),
                    data: &self.weather_data.temp_history,
                },
            ),
            tariffs_buy: Series {
                name: "Tariffs Buy".to_string(),
                chart_type: String::new(),
                data: &self.mygrid_data.tariffs_buy,
            }
        };
        Ok(serde_json::to_string_pretty(&reply)?)
    }

    /// Returns a json object with all necessary data for the full dash
    ///
    fn get_full_dash_data(&self) -> Result<String, DispatcherError> {
        #[derive(Serialize)]
        struct FullDashData<'a> {
            policy: u8,
            temp_current: f64,
            yesterday_min: f64,
            yesterday_max: f64,
            today_min: f64,
            today_max: f64,
            current_prod_load: Series<'a, DataPoint<f64>>,
            current_soc_policy: Series<'a, DataPoint<u8>>,
            tariffs_buy: Series<'a, DataItem<f64>>,
            prod_diagram: (Series<'a, DataItem<f64>>, Series<'a, DataItem<f64>>),
            load_diagram: (Series<'a, DataItem<f64>>, Series<'a, DataItem<f64>>),
            cloud_diagram: Series<'a, DataItem<f64>>,
            temp_diagram: (Series<'a, DataItem<f64>>, Series<'a, DataItem<f64>>),
        }

        let reply = FullDashData {
            policy: self.usage_policy,
            temp_current: self.weather_data.temp_current,
            yesterday_min: self.weather_data.min_max.yesterday_min,
            yesterday_max: self.weather_data.min_max.yesterday_max,
            today_min: self.weather_data.min_max.today_min,
            today_max: self.weather_data.min_max.today_max,
            current_prod_load: Series {
                name: String::new(),
                chart_type: String::new(),
                data: &vec![
                    DataPoint { x: "Production".to_string(), y: self.real_time_data.prod },
                    DataPoint { x: "Load".to_string(), y: self.real_time_data.load }
                ],
            },
            current_soc_policy: Series {
                name: String::new(),
                chart_type: String::new(),
                data: &vec![
                    DataPoint { x: "SoC".to_string(), y: self.real_time_data.soc },
                    DataPoint { x: "Usage Policy".to_string(), y: self.usage_policy }
                ],
            },
            tariffs_buy: Series {
                name: "Tariffs Buy".to_string(),
                chart_type: String::new(),
                data: &self.mygrid_data.tariffs_buy,
            },
            prod_diagram: (
                Series {
                    name: "Estimated Production".to_string(),
                    chart_type: "area".to_string(),
                    data: &self.mygrid_data.prod,
                },
                Series {
                    name: "Production".to_string(),
                    chart_type: "line".to_string(),
                    data: &self.history_data.prod_history,
                },
            ),
            load_diagram: (
                Series {
                    name: "Estimated Load".to_string(),
                    chart_type: "area".to_string(),
                    data: &self.mygrid_data.load,
                },
                Series {
                    name: "Load".to_string(),
                    chart_type: "line".to_string(),
                    data: &self.history_data.load_history,
                },
            ),
            cloud_diagram: Series {
                name: String::new(),
                chart_type: String::new(),
                data: &self.mygrid_data.forecast_cloud,
            },
            temp_diagram: (
                Series {
                    name: "Forecast".to_string(),
                    chart_type: String::new(),
                    data: &self.mygrid_data.forecast_temp,
                },
                Series {
                    name: "Actual".to_string(),
                    chart_type: String::new(),
                    data: &self.weather_data.temp_history,
                },
            ),
        };
        Ok(serde_json::to_string_pretty(&reply)?)
    }

    /// Updates weather data
    /// 
    async fn update_weather(&mut self) -> Result<(), DispatcherError> {
        let local_now = Local::now();
        let utc_now = local_now.with_timezone(&Utc);

        // Check if update is needed
        if self.weather_data.last_end_time.with_timezone(&Local).ordinal0() == local_now.ordinal0() &&
            utc_now - self.weather_data.last_end_time < Duration::minutes(5)
        {
            return Ok(())
        }
        
        info!("updating weather data");
        let from = local_now.duration_trunc(TimeDelta::days(1))?;
        let history = self.weather.get_temp_history(from, local_now, true).await?;
        let last = history.len();
        
        self.weather_data.temp_history = history;
        if last != 0 {
            self.weather_data.temp_current = self.weather_data.temp_history[last - 1].y;
        }
        
        self.weather_data.min_max = self.weather.get_min_max().await?;
        
        self.weather_data.last_end_time = utc_now;
        
        Ok(())
    }
    
    /// Updates all history fields with fresh data, either delta since last update or
    /// from midnight if old data is from yesterday
    /// 
    async fn update_history(&mut self) -> Result<(), DispatcherError> {
        let local_now = Local::now();
        let utc_now = local_now.with_timezone(&Utc);
        
        // Check if update is needed
        if self.history_data.last_end_time.with_timezone(&Local).ordinal0() == local_now.ordinal0() &&
            utc_now - self.history_data.last_end_time <= Duration::minutes(10) 
        {
            return Ok(())
        }

        info!("updating SoC, pvPower and loadsPower history from FoxESS Cloud");
        let mut last_end_time: DateTime<Utc> = utc_now;
        
        let mut start = local_now.duration_trunc(TimeDelta::days(1))?.with_timezone(&Utc);
        if self.history_data.last_end_time.with_timezone(&Local).ordinal0() == local_now.ordinal0() {
            start = self.history_data.last_end_time.add(TimeDelta::seconds(1));
            last_end_time = self.history_data.last_end_time;
        } else {
            self.history_data.soc_history = Vec::new();
            self.history_data.prod_history = Vec::new();
            self.history_data.load_history = Vec::new();
        }

        if utc_now - start >= TimeDelta::minutes(10) {
            let history = self.fox_cloud.get_device_history_data(start, utc_now).await?;
            last_end_time = history.last_end_time;
            
            for (i, time) in history.time.iter().enumerate() {
                let naive_date_time = NaiveDateTime::parse_from_str(time, "%Y-%m-%d %H:%M")?;
                let date_time = naive_date_time.and_local_timezone(Local).unwrap();
                
                self.history_data.soc_history.push(DataItem { x: date_time, y: history.soc[i] });
                self.history_data.prod_history.push(DataItem { x: date_time, y: history.pv_power[i] });
                self.history_data.load_history.push(DataItem { x: date_time, y: history.ld_power[i] });
            }
        }
        
        self.history_data.last_end_time = last_end_time;

        Ok(())
    }

    /// Updates with data from mygrid base data and schedule
    /// 
    /// Base data from mygrid starts with current hour, so the update routine only updates 
    /// current hour and onward to keep an entire day in stock.
    /// 
    async fn update_mygrid_data(&mut self) -> Result<(), DispatcherError> {
        self.schedule =  get_schedule(&self.schedule_path).await?;
        
        let local_now = Local::now();
        let from = local_now.duration_trunc(TimeDelta::days(1))?;
        let to = from.add(TimeDelta::days(1));

        self.mygrid_data = get_base_data(&self.base_data_path, from, to).await?;
            
        Ok(())
    }
    
    /// Updates the real time data field with fresh values
    /// We keep 2 and add the latest to have three values to return as a weighted moving average
    /// 
    /// If the currently stored real time data is older than 10 minutes we start from scratch
    /// 
    async fn update_real_time_data(&mut self) -> Result<(), DispatcherError> {
        let timestamp = Utc::now().timestamp();
        if timestamp - self.real_time_data.timestamp < 180 { return Ok(())}
            
        info!("updating real time data");
        if timestamp - self.real_time_data.timestamp > 600 {
            self.real_time_data.prod_data = VecDeque::new();
            self.real_time_data.load_data = VecDeque::new();
        }

        let real_time_data = self.fox_cloud.get_device_real_time_data().await?;
        self.real_time_data.soc = real_time_data.soc;
        
        if self.real_time_data.prod_data.len() == 3 {
            self.real_time_data.prod_data.pop_front();
        }
        self.real_time_data.prod_data.push_back(real_time_data.pv_power);
        self.real_time_data.prod = two_decimals(get_wma(&self.real_time_data.prod_data));
        
        if self.real_time_data.load_data.len() == 3 {
            self.real_time_data.load_data.pop_front();
        }
        self.real_time_data.load_data.push_back(real_time_data.ld_power);
        self.real_time_data.load = two_decimals(get_wma(&self.real_time_data.load_data));
        
        self.real_time_data.timestamp = timestamp;
        
        Ok(())
    }
    
    /// Evaluates usage policy
    /// 
    fn evaluate_policy(&mut self) -> Result<(), DispatcherError> {
        let data = PolicyData {
            schedule: &self.schedule,
            prod: self.real_time_data.prod,
            load: self.real_time_data.load,
            soc: self.real_time_data.soc,
            policy_tariffs: &self.mygrid_data.policy_tariffs,
            date_time: Local::now().duration_trunc(TimeDelta::hours(1))?,
        };
        
        self.usage_policy = get_policy(data) * 10;
        
        Ok(())
    }
    
    /// Check if it is time to update data from FoxESS
    /// 
    /// # Arguments
    /// 
    /// * 'reset_last_request' - whether to reset or not
    async fn check_updates(&mut self, reset_last_request: bool) -> Result<(), DispatcherError> {
        if reset_last_request {
            self.last_request = Utc::now().timestamp();
        }
        
        if Utc::now().timestamp() - self.last_request <= 1800 {
            let _ = self.update_weather().await?;
            let _ = self.update_real_time_data().await?;
            let _ = self.update_history().await?;
            let _ = self.evaluate_policy()?;
        }
        
        Ok(())
    }
}

/// Returns the weighted moving average from the given vector
/// If the given vector is empty a 0.0 is returned
/// 
/// # Arguments
/// 
/// * 'vec_in' - vector to calculate wma for
fn get_wma(vec_in: &VecDeque<f64>) -> f64 {
    let len = vec_in.len();
    if len != 0 {
        let sum = vec_in
            .iter()
            .enumerate()
            .map(|(i, &d)| (i+1) as f64 * d)
            .sum::<f64>();
        let denom = ((len * len + len) / 2) as f64;
        sum / denom
    } else {
        0.0
    }
}

/// Round to two decimals
/// 
/// # Arguments
/// 
/// * 'a' - value to round
fn two_decimals(a: f64) -> f64 {
    (a * 100.0).round() / 100.0
}
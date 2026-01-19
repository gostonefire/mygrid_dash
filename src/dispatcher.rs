use std::collections::{HashMap, VecDeque};
use std::ops::Add;
use chrono::{DateTime, Duration, DurationRound, Local, NaiveDate, TimeDelta, Timelike, Utc};
use log::{error, info};
use serde::Serialize;
use thiserror::Error;
use tokio::select;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use crate::initialization::Config;
use crate::manager_fox_cloud::{Fox, FoxError};
use crate::manager_mygrid::{get_base_data, get_schedule, MyGridError};
use crate::manager_mygrid::models::Block;
use crate::manager_nordpool::{NordPool, NordPoolError};
use crate::manager_weather::{Weather, WeatherError};
use crate::models::{DataItem, DataPoint, HistoryData, MygridData, RealTimeData, Series, TariffColor, TariffFees, TwoDayMinMax, WeatherData};
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
    if let Err(e) = &disp.update_mygrid_data().await {
        error!("while updating mygrid data: {}", e);
    }

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
                    return Err(DispatcherError::DispatchLoopError("cmd receiver closed unexpectedly".to_string()));
                }
            },
            wake = rx_sleep.recv() => {
                if wake.is_some() {
                    let _ = &disp.check_updates(false).await?;
                    let _ = &disp.update_mygrid_data().await?;
                } else {
                    return Err(DispatcherError::DispatchLoopError("wake receiver closed unexpectedly".to_string()));
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
    nordpool: NordPool,
    schedule_path: String,
    base_data_path: String,
    history_data: HistoryData,
    real_time_data: RealTimeData,
    weather_data: WeatherData,
    today_tariffs: Option<Vec<DataItem<f64>>>,
    tomorrow_tariffs: Option<Vec<DataItem<f64>>>,
    policy_tariffs: HashMap<DateTime<Utc>, f64>,
    max_tariff: u8,
    usage_policy: TariffColor,
    last_request: i64,
    time_delta: TimeDelta,
    version: String,
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
        let nordpool = NordPool::new()?;
        let time_delta = if let Some(debug_run_time) = config.general.debug_run_time {
            Utc::now() - debug_run_time.with_timezone(&Utc)
        } else {
            TimeDelta::seconds(0)
        };
        
        Ok(Self {
            schedule: Vec::new(),
            mygrid_data: MygridData {
                base_cost: 0.0,
                schedule_cost: 0.0,
                forecast_temp: Vec::new(),
                forecast_cloud: Vec::new(),
                prod: Vec::new(),
                load: Vec::new(),
                tariff_fees: TariffFees {
                    variable_fee: 0.0,
                    spot_fee_percentage: 0.0,
                    energy_tax: 0.0,
                    swedish_power_grid: 0.0,
                    balance_responsibility: 0.0,
                    electric_certificate: 0.0,
                    guarantees_of_origin: 0.0,
                    fixed: 0.0,
                },
            },
            fox_cloud,
            weather,
            nordpool,
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
                soh: 0,
                prod: 0.0,
                load: 0.0,
                prod_data: VecDeque::new(),
                load_data: VecDeque::new(),
                timestamp: 0,
            },
            weather_data: WeatherData {
                temp_history: Vec::new(),
                forecast_temp: Vec::new(),
                forecast_symbol: Vec::new(),
                min_max: TwoDayMinMax {
                    yesterday_min: 0.0,
                    yesterday_max: 0.0,
                    today_min: 0.0,
                    today_max: 0.0,
                },
                temp_current: 0.0,
                temp_perceived: 0.0,
                last_end_time: Default::default(),
            },
            today_tariffs: None,
            tomorrow_tariffs: None,
            policy_tariffs: HashMap::new(),
            max_tariff: 0,
            usage_policy: TariffColor::Green,
            last_request: 0,
            time_delta,
            version: config.general.version.clone(),
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
            policy: TariffColor,
            temp_current: f64,
            temp_perceived: f64,
            yesterday_min: f64,
            yesterday_max: f64,
            today_min: f64,
            today_max: f64,
            forecast_symbol: &'a Vec<DataItem<u8>>,
            temp_diagram: (Series<'a, DataItem<f64>>, Series<'a, DataItem<f64>>),
            tariffs_buy: Option<Series<'a, DataItem<f64>>>,
            tariffs_buy_tomorrow: Option<Series<'a, DataItem<f64>>>,
            max_tariff: u8,
            schedule: &'a Vec<Block>,
            base_cost: f64,
            schedule_cost: f64,
            time_delta: i64,
            version: &'a String,
        }

        let tariffs_buy = if let Some(tariffs) = &self.today_tariffs {
            Some(
                Series {
                    name: "Tariffs".to_string(),
                    chart_type: String::new(),
                    data: tariffs,
                }
            )
        } else {
            None
        };

        let tariffs_buy_tomorrow = if let Some(tariffs) = &self.tomorrow_tariffs {
            Some(
                Series {
                    name: "Tariffs".to_string(),
                    chart_type: String::new(),
                    data: tariffs,
                }
            )
        } else {
            None
        };
        
        let reply = SmallDashData {
            policy: self.usage_policy.clone(),
            temp_current: self.weather_data.temp_current,
            temp_perceived: self.weather_data.temp_perceived,
            yesterday_min: self.weather_data.min_max.yesterday_min,
            yesterday_max: self.weather_data.min_max.yesterday_max,
            today_min: self.weather_data.min_max.today_min,
            today_max: self.weather_data.min_max.today_max,
            forecast_symbol: &self.weather_data.forecast_symbol,
            temp_diagram: (
                Series {
                    name: "Forecast".to_string(),
                    chart_type: String::new(),
                    data: &self.weather_data.forecast_temp,
                },
                Series {
                    name: "Actual".to_string(),
                    chart_type: String::new(),
                    data: &self.weather_data.temp_history,
                },
            ),
            tariffs_buy,
            tariffs_buy_tomorrow,
            max_tariff: self.max_tariff,
            schedule: &self.schedule,
            base_cost: self.mygrid_data.base_cost,
            schedule_cost: self.mygrid_data.schedule_cost,
            time_delta: self.time_delta.num_milliseconds(),
            version: &self.version,
        };

        Ok(serde_json::to_string_pretty(&reply)?)
    }

    /// Returns a json object with all necessary data for the full dash
    ///
    fn get_full_dash_data(&self) -> Result<String, DispatcherError> {
        #[derive(Serialize)]
        struct FullDashData<'a> {
            policy: TariffColor,
            temp_current: f64,
            temp_perceived: f64,
            yesterday_min: f64,
            yesterday_max: f64,
            today_min: f64,
            today_max: f64,
            current_prod_load: Series<'a, DataPoint<f64>>,
            current_soc_soh: Series<'a, DataPoint<u8>>,
            tariffs_buy: Option<Series<'a, DataItem<f64>>>,
            max_tariff: u8,
            prod_diagram: (Series<'a, DataItem<f64>>, Series<'a, DataItem<f64>>),
            load_diagram: (Series<'a, DataItem<f64>>, Series<'a, DataItem<f64>>),
            cloud_diagram: Series<'a, DataItem<f64>>,
            temp_diagram: (Series<'a, DataItem<f64>>, Series<'a, DataItem<f64>>),
            time_delta: i64,
        }

        let tariffs_buy = if let Some(tariffs) = &self.today_tariffs {
            Some(
                Series {
                    name: "Tariffs".to_string(),
                    chart_type: String::new(),
                    data: tariffs,
                }
            )
        } else {
            None
        };

        let reply = FullDashData {
            policy: self.usage_policy.clone(),
            temp_current: self.weather_data.temp_current,
            temp_perceived: self.weather_data.temp_perceived,
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
            current_soc_soh: Series {
                name: String::new(),
                chart_type: String::new(),
                data: &vec![
                    DataPoint { x: "SoC".to_string(), y: self.real_time_data.soc },
                    DataPoint { x: "SoH".to_string(), y: self.real_time_data.soh, }
                ],
            },
            tariffs_buy,
            max_tariff: self.max_tariff,
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
                    name: "Forecast (MyGrid)".to_string(),
                    chart_type: String::new(),
                    data: &self.mygrid_data.forecast_temp,
                },
                Series {
                    name: "Actual".to_string(),
                    chart_type: String::new(),
                    data: &self.weather_data.temp_history,
                },
            ),
            time_delta: self.time_delta.num_milliseconds(),
        };
        Ok(serde_json::to_string_pretty(&reply)?)
    }

    /// Updates weather data
    ///
    /// # Arguments
    ///
    /// * 'utc_now' - 'now' according to the Utc timezone
    async fn update_weather(&mut self, utc_now: DateTime<Utc>) -> Result<(), DispatcherError> {
        let (today_start, today_end, _) = get_utc_day_start(utc_now, 0);
        let (yesterday_start, yesterday_end, _) = get_utc_day_start(utc_now, -1);

        // Check if update is needed
        if self.weather_data.last_end_time >= today_start &&  self.weather_data.last_end_time < today_end  &&
            utc_now - self.weather_data.last_end_time < Duration::minutes(5)
        {
            return Ok(())
        }
        
        info!("updating weather data");
        let forecast = self.weather.get_forecast(today_start, today_end).await?;
        self.weather_data.forecast_temp = forecast.forecast_temp;
        self.weather_data.forecast_symbol = forecast.symbol_code;
        
        let history = self.weather.get_temp_history(today_start, utc_now, true).await?;

        self.weather_data.temp_history = history.history;
        self.weather_data.temp_current = history.current_temp.unwrap_or(0.0);
        self.weather_data.temp_perceived = history.perceived_temp.unwrap_or(0.0);


        let (yesterday_min, yesterday_max) = self.weather.get_min_max(yesterday_start, yesterday_end).await?;
        let (today_min, today_max) = self.weather.get_min_max(today_start, today_end).await?;
        self.weather_data.min_max = TwoDayMinMax { yesterday_min, yesterday_max, today_min, today_max };
        
        self.weather_data.last_end_time = utc_now;
        
        Ok(())
    }
    
    /// Updates all history fields with fresh data, either delta since last update or
    /// from midnight if old data is from yesterday
    ///
    /// # Arguments
    ///
    /// * 'utc_now' - 'now' according to the Utc timezone
    async fn update_history(&mut self, utc_now: DateTime<Utc>) -> Result<(), DispatcherError> {
        let (today_start, today_end, _) = get_utc_day_start(utc_now, 0);

        // Check if update is needed
        if self.history_data.last_end_time >= today_start &&  self.history_data.last_end_time < today_end &&
            utc_now - self.history_data.last_end_time <= Duration::minutes(10) 
        {
            return Ok(())
        }

        info!("updating SoC, pvPower and loadsPower history from FoxESS Cloud");
        let mut last_end_time: DateTime<Utc> = utc_now;
        
        let mut start = today_start;
        if self.history_data.last_end_time >= today_start &&  self.history_data.last_end_time < today_end {
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
            
            for (i, &date_time) in history.time.iter().enumerate() {
                self.history_data.soc_history.push(DataItem { x: date_time, y: history.soc[i] });
                self.history_data.prod_history.push(DataItem { x: date_time, y: history.pv_power[i] });
                self.history_data.load_history.push(DataItem { x: date_time, y: history.ld_power[i] });
            }
        }
        
        self.history_data.last_end_time = last_end_time;

        Ok(())
    }

    /// Updates with data from mygrid base data, schedule, and tariffs.
    ///
    async fn update_mygrid_data(&mut self) -> Result<(), DispatcherError> {
        let utc_now = self.utc_now();

        self.schedule =  get_schedule(&self.schedule_path).await?;

        let (day_start, day_end, day_date) = get_utc_day_start(utc_now, 0);
        let (tomorrow_start, tomorrow_end, tomorrow_day_date) = get_utc_day_start(utc_now, 1);

        self.mygrid_data = get_base_data(&self.base_data_path, utc_now, day_start, day_end).await?;
        self.nordpool.set_tariff_fees(self.mygrid_data.tariff_fees.clone());

        self.update_tariffs_if_needed(&self.today_tariffs, day_start, day_end, day_date).await?
            .map(|t| {
                self.today_tariffs = t;
                self.policy_tariffs = self.today_tariffs
                    .as_ref()
                    .map(|v| v.iter()
                        .map(|t| (t.x, t.y))
                        .collect())
                    .unwrap_or_default();
            });

        self.update_tariffs_if_needed(&self.tomorrow_tariffs, tomorrow_start, tomorrow_end, tomorrow_day_date).await?
            .map(|t| self.tomorrow_tariffs = t);

        self.max_tariff = self.max_tariff();

        Ok(())
    }

    /// Updates tariffs if needed
    ///
    /// # Arguments
    ///
    /// * 'tariffs' - tariffs to update if needed
    /// * 'day_start' - start of the day to update tariffs for
    /// * 'day_end' - end of the day to update tariffs for
    /// * 'day_date' - date of the day to update tariffs for
    async fn update_tariffs_if_needed(&self, tariffs: &Option<Vec<DataItem<f64>>>, day_start: DateTime<Utc>, day_end: DateTime<Utc>, day_date: NaiveDate) -> Result<Option<Option<Vec<DataItem<f64>>>>, DispatcherError> {
        let needs_tariff_update = tariffs.as_ref()
            .map(|t| t.first().map_or(true, |d| d.x != day_start))
            .unwrap_or(true);

        if needs_tariff_update {
            let new_tariffs = self.nordpool.get_tariffs(day_start, day_end, day_date).await?
                .inspect(|_| info!("updating tariffs for {}", day_date.format("%Y-%m-%d")));
            
            Ok(Some(new_tariffs))
        } else {
            Ok(None)
        }
    }

    /// Updates the real time data field with fresh values
    /// We keep 2 and add the latest to have three values to return as a weighted moving average
    /// 
    /// If the currently stored real time data is older than 10 minutes we start from scratch
    ///
    /// # Arguments
    ///
    /// * 'utc_now' - 'now' according to the Utc timezone
    async fn update_real_time_data(&mut self, utc_now: DateTime<Utc>) -> Result<(), DispatcherError> {
        let timestamp = utc_now.timestamp();
        if timestamp - self.real_time_data.timestamp < 180 { return Ok(())}
            
        info!("updating real time data");
        if timestamp - self.real_time_data.timestamp > 600 {
            self.real_time_data.prod_data = VecDeque::new();
            self.real_time_data.load_data = VecDeque::new();
        }

        let real_time_data = self.fox_cloud.get_device_real_time_data().await?;
        self.real_time_data.soc = real_time_data.soc;
        self.real_time_data.soh = real_time_data.soh;
        
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
    /// # Arguments
    ///
    /// * 'utc_now' - 'now' according to the Utc timezone
    fn evaluate_policy(&mut self, utc_now: DateTime<Utc>) -> Result<(), DispatcherError> {

        self.usage_policy = get_policy(
            utc_now.duration_trunc(TimeDelta::minutes(15))?, 
            self.real_time_data.soc,
            &self.schedule,
            &self.policy_tariffs,
        );
        
        Ok(())
    }
    
    /// Check if it is time to update data from FoxESS
    /// 
    /// # Arguments
    /// 
    /// * 'reset_last_request' - whether to reset or not
    async fn check_updates(&mut self, reset_last_request: bool) -> Result<(), DispatcherError> {
        let utc_now = self.utc_now();

        if reset_last_request {
            self.last_request = utc_now.timestamp();
        }
        
        if utc_now.timestamp() - self.last_request <= 1800 {
            let _ = self.update_weather(utc_now).await?;
            let _ = self.update_real_time_data(utc_now).await?;
            let _ = self.update_history(utc_now).await?;
            let _ = self.evaluate_policy(utc_now)?;
        }
        
        Ok(())
    }

    /// Calculates max tariff rounded up to the nearest even whole integer value, with a minimum
    /// returned value of 4
    ///
    fn max_tariff(&self) -> u8 {
        let max_today = self.today_tariffs
            .iter()
            .flatten()
            .map(|d| d.y.ceil() as u8)
            .max()
            .unwrap_or(0);

        let max_tomorrow = self.tomorrow_tariffs
            .iter()
            .flatten()
            .map(|d| d.y.ceil() as u8)
            .max()
            .unwrap_or(0);

        let max = max_today.max(max_tomorrow).max(4);

        // Round up to the nearest even whole integer by adding 1 and masking off the lowest bit
        (max + 1) & !1
    }

    /// Returns utc now with any configured time delta applied
    ///
    pub fn utc_now(&self) -> DateTime<Utc> {
        Utc::now() - self.time_delta
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

/// Returns the start and end (non-inclusive) of a day in UTC time.
/// For DST switch days (summer to winter time and vice versa), the length of the day
/// will be either 23 hours (in the spring) or 25 hours (in the autumn).
///
/// It also returns the day date for convenience as a third value in the return tuple
///
/// # Arguments
///
/// * 'date_time' - date time to get utc day start and end for (in relation to Local timezone)
/// * 'day_index' - 0-based index of the day, 0 is today, -1 is yesterday, etc.
fn get_utc_day_start(date_time: DateTime<Utc>, day_index: i64) -> (DateTime<Utc>, DateTime<Utc>, NaiveDate) {
    // First, go local and move hour to a safe place regarding DST day shift between summer and winter time.
    // Also, apply the day index to get to the desired day.
    let date = date_time.with_timezone(&Local).with_hour(12).unwrap().add(TimeDelta::days(day_index));

    // Then trunc to a whole hour and move time to the start of day local (Chrono manages offset change if necessary)
    let start = date.duration_trunc(TimeDelta::hours(1)).unwrap().with_hour(0).unwrap();

    // Then add one day and do the same as for start
    let end = date.add(TimeDelta::days(1)).duration_trunc(TimeDelta::hours(1)).unwrap().with_hour(0).unwrap();

    (start.with_timezone(&Utc), end.with_timezone(&Utc), date.date_naive())
}

#[derive(Debug, Error)]
pub enum DispatcherError {
    #[error("MyGridError: {0}")]
    MyGridError(#[from] MyGridError),
    #[error("SendError: {0}")]
    SendError(#[from] tokio::sync::mpsc::error::SendError<String>),
    #[error("FoxError: {0}")]
    FoxError(#[from] FoxError),
    #[error("ChronoParseError: {0}")]
    ChronoParseError(#[from] chrono::format::ParseError),
    #[error("ChronoRoundingError: {0}")]
    RoundingError(#[from] chrono::round::RoundingError),
    #[error("SerdeJsonError: {0}")]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("WeatherError: {0}")]
    WeatherError(#[from] WeatherError),
    #[error("NordPoolError: {0}")]
    NordPoolError(#[from] NordPoolError),
    #[error("DispatchLoopError: {0}")]
    DispatchLoopError(String),
}

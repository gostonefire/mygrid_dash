use std::collections::VecDeque;
use std::ops::Add;
use chrono::{DateTime, Datelike, Duration, DurationRound, Local, NaiveDateTime, TimeDelta, Utc};
use log::{error, info};
use tokio::select;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use crate::errors::DispatcherError;
use crate::initialization::Config;
use crate::manager_fox_cloud::Fox;
use crate::manager_mygrid::{get_base_data, get_schedule};
use crate::manager_mygrid::errors::MyGridError;
use crate::manager_mygrid::models::Block;
use crate::models::{DataItem, DataPoint, HistoryData, MyGrid, MygridData, RealTimeData, Series};

pub enum Cmd {
    Soc,
    SocHistory,
    Production,
    ProductionHistory,
    Load,
    LoadHistory,
    EstProduction,
    EstLoad,
    CombinedProduction,
    CombinedLoad,
    Schedule,
    ForecastTemp,
    ForecastCloud,
    TariffsBuy,
    TariffsSell,
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
    schedule: Option<Vec<Block>>,
    base_data: MygridData,
    fox_cloud: Fox,
    schedule_path: String,
    base_data_path: String,
    history_data: HistoryData,
    real_time_data: RealTimeData,
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

        Ok(Self {
            schedule: None,
            base_data: MygridData {
                date_time: DateTime::default(),
                forecast_temp: Vec::new(),
                forecast_cloud: Vec::new(),
                production: Vec::new(),
                consumption: Vec::new(),
                tariffs_buy: Vec::new(),
                tariffs_sell: Vec::new(),
            },
            fox_cloud,
            schedule_path: config.mygrid.schedule_path.clone(),
            base_data_path: config.mygrid.base_data_path.clone(),
            history_data: HistoryData {
                soc_history: Vec::new(),
                production_history: Vec::new(),
                load_history: Vec::new(),
                last_end_time: Default::default(),
            },
            real_time_data: RealTimeData {
                soc: 0,
                production: 0.0,
                load: 0.0,
                prod_data: VecDeque::new(),
                load_data: VecDeque::new(),
                timestamp: 0,
            },
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
            Cmd::Soc                 => self.get_current_soc()?,
            Cmd::SocHistory          => self.get_soc_history()?,
            Cmd::Production          => self.get_current_production()?,
            Cmd::ProductionHistory   => self.get_production_history()?,
            Cmd::Load                => self.get_current_load()?,
            Cmd::LoadHistory         => self.get_load_history()?,
            Cmd::EstProduction       => self.get_est_production()?,
            Cmd::EstLoad             => self.get_est_load()?,
            Cmd::CombinedProduction  => self.get_combined_production()?,
            Cmd::CombinedLoad        => self.get_combined_load()?,
            Cmd::Schedule            => self.get_schedule()?,
            Cmd::ForecastTemp        => self.get_forecast_temp()?,
            Cmd::ForecastCloud       => self.get_forecast_cloud()?,
            Cmd::TariffsBuy          => self.get_tariffs_buy()?,
            Cmd::TariffsSell         => self.get_tariffs_sell()?,
        };

        Ok(data)
    }

    /// Returns current SoC
    ///
    fn get_current_soc(&self) -> Result<String, DispatcherError> {
        Ok(serde_json::to_string_pretty(&DataPoint { data: self.real_time_data.soc })?)
    }

    /// Returns soc history since midnight
    /// 
    fn get_soc_history(&self) -> Result<String, DispatcherError> {
        Ok(serde_json::to_string_pretty(&self.history_data.soc_history)?)
    }

    /// Returns the weighted moving average production over the stored real time data points
    ///
    fn get_current_production(&self) -> Result<String, DispatcherError> {
        Ok(serde_json::to_string_pretty(&DataPoint{ data: self.real_time_data.production})?)
    }

    /// Returns production history since midnight
    ///
    fn get_production_history(&self) -> Result<String, DispatcherError> {
        Ok(serde_json::to_string_pretty(&self.history_data.production_history)?)
    }

    /// Returns the weighted moving average load over the stored real time data points
    /// 
    fn get_current_load(&self) -> Result<String, DispatcherError> {
        Ok(serde_json::to_string_pretty(&DataPoint{ data: self.real_time_data.load})?)
    }
    
    /// Returns load history since midnight
    ///
    fn get_load_history(&self) -> Result<String, DispatcherError> {
        Ok(serde_json::to_string_pretty(&self.history_data.production_history)?)
    }

    /// Returns estimated production for the day
    /// 
    fn get_est_production(&self) -> Result<String, DispatcherError> {
        Ok(serde_json::to_string_pretty(&self.base_data.production)?)
    }

    /// Returns estimated load for the day
    ///
    fn get_est_load(&self) -> Result<String, DispatcherError> {
        Ok(serde_json::to_string_pretty(&self.base_data.consumption)?)
    }
    
    /// Returns a combined series of estimated production and production history
    /// 
    fn get_combined_production(&self) -> Result<String, DispatcherError> {
        let series: (Series<f64>, Series<f64>) = (
            Series {
                name: "Estimated Production".to_string(),
                chart_type: "area".to_string(),
                data: &self.base_data.production,
            },            Series {
                name: "Production".to_string(),
                chart_type: "line".to_string(),
                data: &self.history_data.production_history,
            },
        );
     
        Ok(serde_json::to_string_pretty(&series)?)
    }

    /// Returns a combined series of estimated load and load history
    ///
    fn get_combined_load(&self) -> Result<String, DispatcherError> {
        let series: (Series<f64>, Series<f64>) = (
            Series {
                name: "Estimated Load".to_string(),
                chart_type: "area".to_string(),
                data: &self.base_data.consumption,
            },            Series {
                name: "Load".to_string(),
                chart_type: "line".to_string(),
                data: &self.history_data.load_history,
            },
        );

        Ok(serde_json::to_string_pretty(&series)?)
    }

    /// Returns current schedule
    ///
    fn get_schedule(&self) -> Result<String, DispatcherError> {
        Ok(serde_json::to_string_pretty(&self.schedule)?)
    }

    /// Returns current whether forecast temperature
    ///
    fn get_forecast_temp(&self) -> Result<String, DispatcherError> {
        Ok(serde_json::to_string_pretty(&self.base_data.forecast_temp)?)
    }

    /// Returns current whether forecast cloud factor
    ///
    fn get_forecast_cloud(&self) -> Result<String, DispatcherError> {
        Ok(serde_json::to_string_pretty(&self.base_data.forecast_cloud)?)
    }

    /// Returns the buy tariffs for the day
    ///
    fn get_tariffs_buy(&self) -> Result<String, DispatcherError> {
        Ok(serde_json::to_string_pretty(&self.base_data.tariffs_buy)?)
    }

    /// Returns the sell tariffs for the day
    ///
    fn get_tariffs_sell(&self) -> Result<String, DispatcherError> {
        Ok(serde_json::to_string_pretty(&self.base_data.tariffs_sell)?)
    }

    /// Updates all history fields with fresh data, either delta since last update or
    /// from midnight if old data is from yesterday
    /// 
    async fn update_history(&mut self) -> Result<(), DispatcherError> {
        let local_now = Local::now();
        let utc_now = local_now.with_timezone(&Utc);
        
        // Check if update is needed
        if self.history_data.last_end_time.ordinal0() == utc_now.ordinal0() &&
            utc_now - self.history_data.last_end_time <= Duration::minutes(10) 
        {
            return Ok(())
        }

        info!("updating SoC, pvPower and loadsPower history from FoxESS Cloud");
        let mut last_end_time: DateTime<Utc> = utc_now;
        
        let mut start = local_now.duration_trunc(TimeDelta::days(1))?.with_timezone(&Utc);
        if self.history_data.last_end_time.ordinal0() == utc_now.ordinal0() {
            start = self.history_data.last_end_time.add(TimeDelta::seconds(1));
            last_end_time = self.history_data.last_end_time;
        } else {
            self.history_data.soc_history = Vec::new();
            self.history_data.production_history = Vec::new();
            self.history_data.load_history = Vec::new();
        }

        if utc_now - start >= TimeDelta::minutes(10) {
            let history = self.fox_cloud.get_device_history_data(start, utc_now).await?;
            last_end_time = history.last_end_time;
            
            for (i, time) in history.time.iter().enumerate() {
                let naive_date_time = NaiveDateTime::parse_from_str(time, "%Y-%m-%d %H:%M")?;
                //let date_time: DateTime<Local> = naive_date_time.and_utc().with_timezone(&Local);
                let date_time = naive_date_time.and_local_timezone(Local).unwrap();
                
                self.history_data.soc_history.push(DataItem { x: date_time, y: history.soc[i] });
                self.history_data.production_history.push(DataItem { x: date_time, y: history.pv_power[i] });
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
        self.schedule =  Some(get_schedule(&self.schedule_path).await?);
        let base_data = get_base_data(&self.base_data_path).await?;
        let local_now = Local::now();
        let from = local_now.duration_trunc(TimeDelta::days(1))?;
        let to = from.add(TimeDelta::days(1));
        
        let mut forecast_temp = filter_time(&base_data.forecast_temp, from, to);
        let mut forecast_cloud = filter_time(&base_data.forecast_cloud, from, to);
        let mut production = filter_time(&base_data.production, from, to);
        let mut consumption = filter_time(&base_data.consumption, from, to);
        let mut tariffs_buy = filter_time(&base_data.tariffs_buy, from, to);
        let mut tariffs_sell = filter_time(&base_data.tariffs_sell, from, to);

        let load_start = base_data.date_time.duration_trunc(TimeDelta::hours(1))?;
        
        if self.base_data.date_time.ordinal0() == base_data.date_time.ordinal0() {
            forecast_temp = append_tail(
                filter_time(&self.base_data.forecast_temp, from, load_start), 
                forecast_temp
            );

            forecast_cloud = append_tail(
                filter_time(&self.base_data.forecast_cloud, from, load_start),
                forecast_cloud
            );

            production = append_tail(
                filter_time(&self.base_data.production, from, load_start),
                production
            );

            consumption = append_tail(
                filter_time(&self.base_data.consumption, from, load_start),
                consumption
            );

            tariffs_buy = append_tail(
                filter_time(&self.base_data.tariffs_buy, from, load_start),
                tariffs_buy
            );

            tariffs_sell = append_tail(
                filter_time(&self.base_data.tariffs_sell, from, load_start),
                tariffs_sell
            );
        };
        
        self.base_data = MygridData {
            date_time: base_data.date_time,
            forecast_temp: pad(forecast_temp, DataItem { x: local_now, y: 0.0 })?,
            forecast_cloud: pad(forecast_cloud, DataItem { x: local_now, y: 0.0 })?,
            production: pad(production, DataItem { x: local_now, y: 0.0 })?,
            consumption: pad(consumption, DataItem { x: local_now, y: 0.0 })?,
            tariffs_buy: pad(tariffs_buy, DataItem { x: local_now, y: 0.0 })?,
            tariffs_sell: pad(tariffs_sell, DataItem { x: local_now, y: 0.0 })?,
        };
            
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
        self.real_time_data.production = get_wma(&self.real_time_data.prod_data);
        
        if self.real_time_data.load_data.len() == 3 {
            self.real_time_data.load_data.pop_front();
        }
        self.real_time_data.load_data.push_back(real_time_data.ld_power);
        self.real_time_data.load = get_wma(&self.real_time_data.load_data);
        
        self.real_time_data.timestamp = timestamp;
        
        Ok(())
    }
    
    /// Check if it is time to update data from FoxESS
    /// 
    /// # Arguments
    /// 
    /// * 'reset_last_request' - whether to reset or not
    async fn check_updates(&mut self, reset_last_request: bool) -> Result<(), DispatcherError> {
        info!("checking for FoxESS updates");
        if reset_last_request {
            self.last_request = Utc::now().timestamp();
        }
        
        if Utc::now().timestamp() - self.last_request <= 1800 {
            let _ = self.update_real_time_data().await?;
            let _ = self.update_history().await?;
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

/// Returns a vector with items from the given vector where the date is greater or equal to start
/// end less than end
///
/// # Arguments
///
/// * 'vec_in' - the input vector to filter
/// * 'start' - the start date to compare with
/// * 'end' - the end date to compare with 
fn filter_time<T: MyGrid + Clone>(vec_in: &Vec<T>, start: DateTime<Local>, end: DateTime<Local>) -> Vec<T> {
    vec_in
        .iter()
        .filter(|f| f.is_within(start, end))
        .map(|f| f.clone())
        .collect::<Vec<T>>()
}

/// Pads (left) with missing hours from midnight
/// The value field is set according the given model
/// 
/// # Arguments
/// 
/// * 'vec_in' - the vector to pad
/// * 'model' - the model struct which has the date field set to today's date and value fields according to what the padded dates should be set to
fn pad<T: MyGrid<Item = T>>(mut vec_in: Vec<T>, model: T) -> Result<Vec<T>, MyGridError> {
    let start = model.date_time_day()?;
    let mut end = if let Some(t) = vec_in.get(0) {
        t.date_time_hour()?
    } else {
        start.add(TimeDelta::days(1))
    };

    while start < end {
        end += TimeDelta::hours(-1);
        vec_in.insert(0, model.create_new(end));
    }

    Ok(vec_in)
}

/// Appends a vector
/// 
/// # Arguments
/// 
/// * 'this' - the vector to append to
/// * 'other' - the vector to append 
fn append_tail<T: MyGrid<Item = T>>(mut this: Vec<T>, mut other: Vec<T>) -> Vec<T> {
    this.append(&mut other);
    this
}
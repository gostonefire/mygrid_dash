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
use crate::manager_mygrid::models::{BaseData, Block};

#[derive(Debug)]
pub enum Cmd {
    Soc(Option<String>),
    SocHistory(Option<String>),
    Production(Option<String>),
    ProductionHistory(Option<String>),
    Load(Option<String>),
    LoadHistory(Option<String>),
    EstProduction(Option<String>),
    EstLoad(Option<String>),
    Schedule(Option<String>),
    Forecast(Option<String>),
    Tariffs(Option<String>),
    NoOp,
}

#[derive(Serialize)]
struct Soc {
    soc: u8,
    #[serde(skip)]
    timestamp: i64,
}

#[derive(Serialize)]
struct History<T> {
    date_time: Vec<DateTime<Local>>,
    data: Vec<T>,
}

pub struct HistoryData {
    soc_history: History<u8>,
    production_history: History<f64>,
    load_history: History<f64>,
    last_history_time: DateTime<Local>
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
    let mut disp = match Dispatcher::new(tx, rx, config).await {
        Ok(d) => d,
        Err(e) => { error!("while initializing dispatcher: {}", e); return; }
    };

    match disp.dispatch_loop().await {
        Ok(_) => {
            info!("dispatch loop terminated");
        },
        Err(e) => {
            error!("dispatch loop terminated with error: {}", e);
        }
    }
}

/// Dispatcher struct
///
struct Dispatcher {
    tx: UnboundedSender<String>,
    rx: UnboundedReceiver<Cmd>,
    schedule: Vec<Block>,
    base_data: BaseData,
    fox_cloud: Fox,
    schedule_path: String,
    base_data_path: String,
    current_soc: Option<Soc>,
    history_data: Option<HistoryData>,
}

impl Dispatcher {
    /// Creates a new `Dispatcher` ready for action
    ///
    /// # Arguments
    ///
    /// * 'tx' - mpsc sender to the web server
    /// * 'rx' - mpsc receiver from the web server
    /// * 'config' - configuration struct
    async fn new(tx: UnboundedSender<String>, rx: UnboundedReceiver<Cmd>, config: &Config) -> Result<Self, DispatcherError> {
        let schedule = get_schedule(&config.mygrid.schedule_path).await?;
        let base_data = get_base_data(&config.mygrid.base_data_path).await?;
        let fox_cloud = Fox::new(&config.fox_ess)?;

        Ok(Self {
            tx,
            rx,
            schedule,
            base_data,
            fox_cloud,
            schedule_path: config.mygrid.schedule_path.clone(),
            base_data_path: config.mygrid.base_data_path.clone(),
            current_soc: None,
            history_data: None,
        })
    }

    /// Main dispatch loop that regularly read mygrid files and builds up history data
    /// while also listening for requests from the web server
    ///
    async fn dispatch_loop(&mut self) -> Result<(), DispatcherError> {
        loop {
            select! {
            cmd = self.rx.recv() => {
                if let Some(cmd) = cmd {
                    let data = self.execute_cmd(cmd).await?;
                    self.tx.send(data)?;
                } else {
                    return Err("receiver closed unexpectedly".into());
                }
            }
            _ = tokio::time::sleep(tokio::time::Duration::from_secs(300)) => {

            }
        }
        }
    }

    /// Executes a command and returns the same command but with requested data
    ///
    /// # Arguments
    ///
    /// * 'cmd' - the command to evaluate and execute
    async fn execute_cmd(&mut self, cmd: Cmd) -> Result<String, DispatcherError> {
        let data = match cmd {
            Cmd::Soc(_)               => self.get_current_soc().await?,
            Cmd::SocHistory(_)        => self.get_soc_history().await?,
            Cmd::Production(_)        => String::new(),
            Cmd::ProductionHistory(_) => self.get_production_history().await?,
            Cmd::Load(_)              => String::new(),
            Cmd::LoadHistory(_)       => self.get_load_history().await?,
            Cmd::EstProduction(_)     => self.get_est_production()?,
            Cmd::EstLoad(_)           => self.get_est_load()?,
            Cmd::Schedule(_)          => self.get_schedule()?,
            Cmd::Forecast(_)          => self.get_forecast()?,
            Cmd::Tariffs(_)           => self.get_tariffs()?,
            Cmd::NoOp                 => String::new(),
        };

        Ok(data)
    }

    /// Returns current SoC
    /// If the currently stored SoC is None or to old, a fresh SoC is fetched from FoxESS Cloud
    ///
    async fn get_current_soc(&mut self) -> Result<String, DispatcherError> {
        if self.current_soc.is_none() || self.current_soc.as_ref().is_some_and(|s| Utc::now().timestamp() - s.timestamp > 300) {
            let soc = self.fox_cloud.get_current_soc().await?;
            self.current_soc = Some(Soc {
                soc,
                timestamp: Utc::now().timestamp(),
            })
        }

        Ok(serde_json::to_string_pretty(&self.current_soc.as_ref().unwrap())?)
    }

    /// Returns soc history since midnight
    /// If needed the history is first updated from FoxESS Cloud
    /// 
    async fn get_soc_history(&mut self) -> Result<String, DispatcherError> {
        self.update_history().await?;

        Ok(serde_json::to_string_pretty(&self.history_data.as_ref().unwrap().soc_history)?)
    }

    /// Returns production history since midnight
    /// If needed the history is first updated from FoxESS Cloud
    ///
    async fn get_production_history(&mut self) -> Result<String, DispatcherError> {
        self.update_history().await?;

        Ok(serde_json::to_string_pretty(&self.history_data.as_ref().unwrap().production_history)?)
    }

    /// Returns load history since midnight
    /// If needed the history is first updated from FoxESS Cloud
    ///
    async fn get_load_history(&mut self) -> Result<String, DispatcherError> {
        self.update_history().await?;

        Ok(serde_json::to_string_pretty(&self.history_data.as_ref().unwrap().production_history)?)
    }

    /// Returns estimated production for the day
    /// 
    fn get_est_production(&mut self) -> Result<String, DispatcherError> {
        Ok(serde_json::to_string_pretty(&self.base_data.production)?)
    }

    /// Returns estimated load for the day
    ///
    fn get_est_load(&mut self) -> Result<String, DispatcherError> {
        Ok(serde_json::to_string_pretty(&self.base_data.consumption)?)
    }

    /// Returns current schedule
    ///
    fn get_schedule(&mut self) -> Result<String, DispatcherError> {
        Ok(serde_json::to_string_pretty(&self.schedule)?)
    }

    /// Returns current whether forecast
    ///
    fn get_forecast(&mut self) -> Result<String, DispatcherError> {
        Ok(serde_json::to_string_pretty(&self.base_data.forecast)?)
    }

    /// Returns tariffs for the day
    ///
    fn get_tariffs(&mut self) -> Result<String, DispatcherError> {
        Ok(serde_json::to_string_pretty(&self.base_data.tariffs)?)
    }

    /// Updates all history fields with fresh data, either delta since last update or
    /// from midnight if old data is from yesterday
    /// 
    async fn update_history(&mut self) -> Result<(), DispatcherError> {
        // Check if update is needed
        if self.history_data
            .as_ref()
            .is_some_and(|l| {
                Utc::now() - l.last_history_time.with_timezone(&Utc) <= Duration::minutes(5) && 
                    l.last_history_time.ordinal0() == Local::now().ordinal0()
            }){
            
            return Ok(())
        }
        
        let mut soc_history: History<u8> = History { date_time: Vec::new(), data: Vec::new() };
        let mut production_history: History<f64> = History { date_time: Vec::new(), data: Vec::new() };
        let mut load_history: History<f64> = History { date_time: Vec::new(), data: Vec::new() };
        let mut last_history_time: DateTime<Local> = Local::now();
        
        let mut start = Local::now().duration_trunc(TimeDelta::days(1))?.with_timezone(&Utc);
        if let Some(hd) = &self.history_data {
            if hd.last_history_time.ordinal0() == Local::now().ordinal0() {
                let hd = self.history_data.take().unwrap(); 
                start = hd.last_history_time.add(TimeDelta::seconds(1)).with_timezone(&Utc);
                soc_history = hd.soc_history;
                production_history = hd.production_history;
                load_history = hd.load_history;
                last_history_time = hd.last_history_time;
            }
        }
        let end =  Local::now().with_timezone(&Utc);

        if end - start >= TimeDelta::minutes(10) {
            let history = self.fox_cloud.get_device_history_data(start, end).await?;
            
            for (i, time) in history.time.iter().enumerate() {
                let naive_date_time = NaiveDateTime::parse_from_str(time, "%Y-%m-%d %H:%M")?;
                let date_time = naive_date_time.and_local_timezone(Local).unwrap();
                
                soc_history.data.push(history.soc[i]);
                soc_history.date_time.push(date_time);

                production_history.data.push(history.pv_power[i]);
                production_history.date_time.push(date_time);

                load_history.data.push(history.ld_power[i]);
                load_history.date_time.push(date_time);

                last_history_time = date_time;
            }
        }

        self.history_data = Some(HistoryData {
            soc_history,
            production_history,
            load_history,
            last_history_time,
        });

        Ok(())
    }
}



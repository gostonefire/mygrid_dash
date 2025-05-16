use std::ops::Add;
use actix_web::cookie::time::format_description::well_known::iso8601::FormattedComponents::Time;
use chrono::{DateTime, Datelike, Duration, DurationRound, Local, NaiveDateTime, TimeDelta, Utc};
use log::{error, info};
use tokio::select;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use crate::errors::DispatcherError;
use crate::initialization::Config;
use crate::manager_fox_cloud::Fox;
use crate::manager_mygrid::{get_base_data, get_schedule};
use crate::manager_mygrid::models::{BaseData, Block};

pub enum Cmd {
    Soc(Option<u8>),
    SocHistory(Option<Vec<DatedData>>),
    Production(Option<f64>),
    ProductionHistory(Option<Vec<DatedData>>),
    Load(Option<f64>),
    LoadHistory(Option<Vec<DatedData>>),
    EstProduction(Option<Vec<DatedData>>),
    EstLoad(Option<Vec<DatedData>>),
    Schema(Option<Vec<Block>>),
    NoOp,
}

#[derive(Clone)]
pub struct DatedData {
    pub date_time: DateTime<Local>,
    pub timestamp: i64,
    pub data: f64,
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
pub async fn run(tx: UnboundedSender<Cmd>,  rx: UnboundedReceiver<Cmd>, config: &Config) {
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
    tx: UnboundedSender<Cmd>,
    rx: UnboundedReceiver<Cmd>,
    schedule: Vec<Block>,
    base_data: BaseData,
    fox_cloud: Fox,
    schedule_path: String,
    base_data_path: String,
    current_soc: Option<DatedData>,
    soc_history: Option<Vec<DatedData>>,
    production_history: Option<Vec<DatedData>>,
    load_history: Option<Vec<DatedData>>,
    last_history_time: Option<DateTime<Local>>
}

impl Dispatcher {
    /// Creates a new `Dispatcher` ready for action
    ///
    /// # Arguments
    ///
    /// * 'tx' - mpsc sender to the web server
    /// * 'rx' - mpsc receiver from the web server
    /// * 'config' - configuration struct
    async fn new(tx: UnboundedSender<Cmd>, rx: UnboundedReceiver<Cmd>, config: &Config) -> Result<Self, DispatcherError> {
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
            soc_history: None,
            production_history: None,
            load_history: None,
            last_history_time: None,
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
    async fn execute_cmd(&mut self, cmd: Cmd) -> Result<Cmd, DispatcherError> {
        let mut data: Cmd = Cmd::NoOp;

        match cmd {
            Cmd::Soc(_) => { data = Cmd::Soc(Some(self.get_current_soc().await?)) }
            Cmd::SocHistory(_) => { data = Cmd::SocHistory(Some(self.get_soc_history().await?)) }
            Cmd::Production(_) => {}
            Cmd::ProductionHistory(_) => { data = Cmd::ProductionHistory(Some(self.get_production_history().await?)) }
            Cmd::Load(_) => {}
            Cmd::LoadHistory(_) => { data = Cmd::LoadHistory(Some(self.get_load_history().await?)) }
            Cmd::EstProduction(_) => {}
            Cmd::EstLoad(_) => {}
            Cmd::Schema(_) => {}
            Cmd::NoOp => ()
        }

        Ok(data)
    }

    /// Returns current SoC
    /// If the currently stored SoC is None or to old, a fresh SoC is fetched from FoxESS Cloud
    ///
    async fn get_current_soc(&mut self) -> Result<u8, DispatcherError> {
        if self.current_soc.is_none() || self.current_soc.as_ref().is_some_and(|s| Utc::now().timestamp() - s.timestamp > 300) {
            let soc = self.fox_cloud.get_current_soc().await?;
            self.current_soc = Some(DatedData {
                date_time: Local::now(),
                timestamp: Utc::now().timestamp(),
                data: soc as f64,
            })
        }

        Ok(self.current_soc.as_ref().map(|s| s.data as u8).unwrap())
    }

    /// Returns soc history since midnight
    /// If needed the history is first updated from FoxESS Cloud
    /// 
    async fn get_soc_history(&mut self) -> Result<Vec<DatedData>, DispatcherError> {
        self.update_history().await?;

        Ok(self.soc_history.as_ref().unwrap().clone())
    }

    /// Returns production history since midnight
    /// If needed the history is first updated from FoxESS Cloud
    ///
    async fn get_production_history(&mut self) -> Result<Vec<DatedData>, DispatcherError> {
        self.update_history().await?;

        Ok(self.production_history.as_ref().unwrap().clone())
    }

    /// Returns load history since midnight
    /// If needed the history is first updated from FoxESS Cloud
    ///
    async fn get_load_history(&mut self) -> Result<Vec<DatedData>, DispatcherError> {
        self.update_history().await?;

        Ok(self.load_history.as_ref().unwrap().clone())
    }

    /// Updates all history fields with fresh data, either delta since last update or
    /// from midnight if old data is from yesterday
    /// 
    async fn update_history(&mut self) -> Result<(), DispatcherError> {
        // Check if update is needed
        if self.last_history_time
            .as_ref()
            .is_some_and(|l| {
                Utc::now() - l.with_timezone(&Utc) <= Duration::minutes(5) && l.ordinal0() == Local::now().ordinal0()
            }){
            
            return Ok(())
        }
        
        let mut soc_history: Vec<DatedData> = Vec::new();
        let mut production_history: Vec<DatedData> = Vec::new();
        let mut load_history: Vec<DatedData> = Vec::new();
        let mut last_history_time: Option<DateTime<Local>> = None;
        
        let mut start: DateTime<Utc> = Local::now().duration_trunc(TimeDelta::days(1)).unwrap().with_timezone(&Utc);
        if let Some(lht) = self.last_history_time {
            if lht.ordinal0() == Local::now().ordinal0() {
                start = lht.add(TimeDelta::seconds(1)).with_timezone(&Utc);
                soc_history = self.soc_history.take().unwrap();
                production_history = self.production_history.take().unwrap();
                load_history = self.load_history.take().unwrap();
                last_history_time = Some(lht);
            }
        }
        let end =  Local::now().with_timezone(&Utc);

        if end - start < TimeDelta::minutes(10) {
            self.soc_history = Some(soc_history);
            self.production_history = Some(production_history);
            self.load_history = Some(load_history);
            self.last_history_time = last_history_time;
        } else {
            let history = self.fox_cloud.get_device_history_data(start, end).await?;
            
            for (i, time) in history.time.iter().enumerate() {
                let naive_date_time = NaiveDateTime::parse_from_str(time, "%Y-%m-%d %H:%M:%S")?;
                let timestamp = naive_date_time.and_utc().timestamp();
                let date_time = naive_date_time.and_local_timezone(Local).unwrap();
                
                soc_history.push(DatedData {
                    date_time,
                    timestamp,
                    data: history.soc[i] as f64,
                });
                production_history.push(DatedData {
                    date_time,
                    timestamp,
                    data: history.pv_power[i],
                });
                load_history.push(DatedData {
                    date_time,
                    timestamp,
                    data: history.ld_power[i],
                });
                last_history_time = Some(date_time);
            }
            self.soc_history = Some(soc_history);
            self.production_history = Some(production_history);
            self.load_history = Some(load_history);
            self.last_history_time = last_history_time;
        }

        Ok(())
    }
}



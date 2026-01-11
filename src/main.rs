extern crate alloc;

use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::FromStr;
use std::sync::Arc;
use axum::http::{header, HeaderValue};
use axum::Router;
use axum::routing::get;
use tokio::sync::{Mutex, RwLock};
use chrono::Utc;
use log::{error, info};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tower_http::services::{ServeDir, ServeFile};
use tower_http::set_header::SetResponseHeaderLayer;
use crate::errors::UnrecoverableError;
use crate::initialization::{config, Google};
use crate::dispatcher::{run, Cmd};
use crate::handlers::*;
use crate::manager_tokens::{google_base_data, Tokens};

mod errors;
mod initialization;
mod logging;
mod manager_fox_cloud;
mod manager_mygrid;
mod dispatcher;
mod handlers;
mod models;
mod usage_policy;
mod manager_weather;
mod manager_tokens;
mod manager_nordpool;

type SessionStore = Arc<RwLock<HashMap<String, (i64, String, Option<Tokens>)>>>;

struct Comms {
    tx_to_mygrid: UnboundedSender<Cmd>,
    rx_from_mygrid: UnboundedReceiver<String>,
}

#[derive(Clone)]
struct AppState {
    comms: Arc<Mutex<Comms>>,
    sessions: SessionStore,
    config: Arc<RwLock<Google>>,
}

#[tokio::main]
async fn main() -> Result<(), UnrecoverableError> {
    // Set up communication channels
    let (mut tx_to_mygrid, mut rx_from_web) = mpsc::unbounded_channel::<Cmd>();
    let (mut tx_to_web, mut rx_from_mygrid) = mpsc::unbounded_channel::<String>();
    let comms = Arc::new(Mutex::new(Comms{tx_to_mygrid,rx_from_mygrid,}));
    
    // Load configuration
    let config = config()?;
    let google_config = Arc::new(RwLock::new(config.google.clone()));
    let session_store: SessionStore = Arc::new(RwLock::new(HashMap::new()));

    // Print version
    info!("mygrid_dash version: {}", config.general.version);

    // Enrich config
    google_base_data(google_config.clone()).await.expect("google base data update should succeed");

    // Purging of old sessions
    info!("starting sessions purge job");
    tokio::spawn(purge_sessions(session_store.clone()));

    // Purging of old sessions
    info!("starting google base data update job");
    tokio::spawn(update_google_base_data(google_config.clone()));

    // Web server
    info!("starting web server");
    let static_service = ServeDir::new("static").append_index_html_on_directories(true);
    let shared_state = AppState {comms: comms.clone(), sessions: session_store.clone(), config: google_config.clone() };

    let app = Router::new()
        .route("/data/{dash_type}", get(get_data))
        .route("/login", get(login))
        .route("/code", get(code))
        .route_service("/full", ServeFile::new("static/index_full.html"))
        .fallback_service(static_service)
        .layer(SetResponseHeaderLayer::if_not_present(
            header::CACHE_CONTROL,
            HeaderValue::from_static("no-cache"),
        ))
        .with_state(shared_state);

    let ip_addr = Ipv4Addr::from_str(&config.web_server.bind_address).expect("invalid BIND_ADDR");
    let addr = SocketAddr::new(IpAddr::V4(ip_addr), config.web_server.bind_port);

    tokio::spawn(axum_server::bind(addr)
        .serve(app.into_make_service()));

    // Main dispatch function
    info!("starting main dispatch function");
    loop {
        run(tx_to_web, rx_from_web, &config).await;

        info!("restarting main dispatch function");
        (tx_to_mygrid, rx_from_web) = mpsc::unbounded_channel::<Cmd>();
        (tx_to_web, rx_from_mygrid) = mpsc::unbounded_channel::<String>();
        {
            let mut disp_comms = comms.lock().await;
            disp_comms.tx_to_mygrid = tx_to_mygrid;
            disp_comms.rx_from_mygrid = rx_from_mygrid;
        }
    }
}

/// Loop that purges old entries from the session store
///
/// # Arguments
///
/// * 'session_store' - the session store
async fn purge_sessions(session_store: SessionStore) {
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
        let limit = Utc::now().timestamp() - 86400;
        {
            let mut sessions = session_store.write().await;
            let keys = sessions.keys().map(|k| k.clone()).collect::<Vec<String>>();
            keys.into_iter().for_each(|k| {
                if sessions.get(&k).is_some_and(|v| v.0 < limit) {
                    sessions.remove(&k);
                }
            });
        }
    }
}

/// Periodically updates google base data such as well known urls and jwks
///
/// # Arguments
///
/// * 'google_config' - google configuration data
async fn update_google_base_data(google_config: Arc<RwLock<Google>>) {
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        if let Err(e) = google_base_data(google_config.clone()).await {
            error!("error in google_base_data: {}", e);
        }
    }
}
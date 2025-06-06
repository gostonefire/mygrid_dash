use std::sync::Arc;
use tokio::sync::Mutex;
use actix_files::Files;
use actix_web::{middleware, web, App, HttpServer};
use log::info;
use rustls::ServerConfig;
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use rustls_pki_types::pem::PemObject;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use crate::errors::UnrecoverableError;
use crate::initialization::{config, WebServerParameters};
use crate::dispatcher::{run, Cmd};
use crate::handlers::*;

mod errors;
mod initialization;
mod logging;
mod manager_fox_cloud;
mod manager_mygrid;
mod dispatcher;
mod handlers;
mod serialize_timestamp;
mod models;
mod usage_policy;
mod traits;

struct Comms {
    tx_to_mygrid: UnboundedSender<Cmd>,
    rx_from_mygrid: UnboundedReceiver<String>,
}

struct AppState {
    comms: Arc<Mutex<Comms>>,
}

#[actix_web::main]
async fn main() -> Result<(), UnrecoverableError> {
    // Set up communication channels
    let (mut tx_to_mygrid, mut rx_from_web) = mpsc::unbounded_channel::<Cmd>();
    let (mut tx_to_web, mut rx_from_mygrid) = mpsc::unbounded_channel::<String>();
    let comms = Arc::new(Mutex::new(Comms{tx_to_mygrid,rx_from_mygrid,}));
    
    // Load configuration
    let config = config()?;

    // Web server
    info!("starting web server");
    let web_comms = comms.clone();
    let rustls_config = load_rustls_config(&config.web_server)?;
    tokio::spawn(HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(AppState {comms: web_comms.clone()}))
                .service(combined_realtime)
                .service(combined_production)
                .service(combined_load)
                .service(schedule)
                .service(forecast_temp)
                .service(forecast_cloud)
                .service(tariffs_buy)
                .service(
                    web::scope("")
                        .wrap(middleware::DefaultHeaders::new().add(("Cache-Control", "no-cache")))
                        .service(Files::new("/short", "./static").index_file("index_short.html"))
                        .service(Files::new("/", "./static").index_file("index.html"))
                )
        })
            .bind_rustls_0_23((config.web_server.bind_address.as_str(), config.web_server.bind_port), rustls_config)?
            //.bind(("127.0.0.1", 8080))?
            .disable_signals()
            .run());

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

/// Loads TLS certificates
///
/// # Arguments
///
/// * 'config' - web server parameters
fn load_rustls_config(config: &WebServerParameters) -> Result<ServerConfig, UnrecoverableError> {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .unwrap();

    // load TLS key/cert files
    let cert_chain = CertificateDer::pem_file_iter(&config.tls_chain_cert)?
        .flatten()
        .collect();

    let key_der =
        PrivateKeyDer::from_pem_file(&config.tls_private_key).expect("Could not locate PKCS 8 private keys.");

    Ok(ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert_chain, key_der)?)
}

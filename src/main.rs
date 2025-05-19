use std::sync::{Arc, Mutex};
use actix_web::{web, App, HttpServer};
use log::info;
use rustls::ServerConfig;
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use rustls_pki_types::pem::PemObject;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use crate::errors::UnrecoverableError;
use crate::initialization::{config, WebServerParameters};
use crate::dispatcher::{run, Cmd};
use crate::handlers::{est_production, soc_current, soc_history};

mod errors;
mod initialization;
mod logging;
mod manager_fox_cloud;
mod manager_mygrid;
mod dispatcher;
mod handlers;

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
    let (tx_to_mygrid, rx_from_web) = mpsc::unbounded_channel::<Cmd>();
    let (tx_to_web, rx_from_mygrid) = mpsc::unbounded_channel::<String>();
    let comms = Arc::new(Mutex::new(Comms{tx_to_mygrid,rx_from_mygrid,}));
    
    // Load configuration
    let config = config()?;

    // Main dispatch function
    info!("starting main dispatch function");
    let c = config.clone();
    tokio::spawn(async move { run(tx_to_web, rx_from_web, &c).await });
    
    // Web server
    info!("starting web server");
    let rustls_config = load_rustls_config(&config.web_server)?;
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {comms: comms.clone()}))
            .service(soc_history)
            .service(soc_current)
            .service(est_production)
    })
        .workers(4)
        .bind_rustls_0_23((config.web_server.bind_address.as_str(), config.web_server.bind_port), rustls_config)?
        .run()
        .await?;

    Ok(())

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

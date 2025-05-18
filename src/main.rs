use std::sync::{Arc, Mutex};
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use log::info;
use rustls::ServerConfig;
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use rustls_pki_types::pem::PemObject;
use serde::Deserialize;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use crate::errors::UnrecoverableError;
use crate::initialization::{config, Config, WebServerParameters};
use crate::dispatcher::{run, Cmd};

mod errors;
mod initialization;
mod logging;
mod manager_fox_cloud;
mod manager_mygrid;
mod dispatcher;

#[derive(Deserialize)]
struct Params {
    code: String,
}

struct Comms {
    tx_to_mygrid: UnboundedSender<Cmd>,
    rx_from_mygrid: UnboundedReceiver<String>,
}

struct AppState {
    comms: Arc<Mutex<Comms>>,
}

#[get("/code")]
async fn code(data: web::Data<AppState>, params: web::Query<Params>) -> impl Responder {
    HttpResponse::Ok().body("Access granted!")
}

#[get("/soc_history")]
async fn soc_history(data: web::Data<AppState>) -> impl Responder {
    let mut comms = data.comms.lock().unwrap();
    comms.tx_to_mygrid.send(Cmd::SocHistory(None)).unwrap();

    if let Some(json) = comms.rx_from_mygrid.recv().await {
        HttpResponse::Ok().body(json)             
    } else {
        HttpResponse::NoContent().finish()
    }
}

#[get("/soc_current")]
async fn soc_current(data: web::Data<AppState>) -> impl Responder {
    let mut comms = data.comms.lock().unwrap();
    comms.tx_to_mygrid.send(Cmd::Soc(None)).unwrap();

    if let Some(json) = comms.rx_from_mygrid.recv().await {
        HttpResponse::Ok().body(json)
    } else {
        HttpResponse::NoContent().finish()
    }
}

#[get("/est_production")]
async fn est_production(data: web::Data<AppState>) -> impl Responder {
    let mut comms = data.comms.lock().unwrap();
    comms.tx_to_mygrid.send(Cmd::EstProduction(None)).unwrap();

    if let Some(json) = comms.rx_from_mygrid.recv().await {
        HttpResponse::Ok().body(json)
    } else {
        HttpResponse::NoContent().finish()
    }
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
    
    // Authentication/authorization function
    info!("starting authentication/authorization function");
    let rustls_config = load_rustls_config(&config.web_server)?;
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {comms: comms.clone()}))
            .service(code)
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

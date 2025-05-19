use actix_web::{get, web, HttpResponse, Responder};
use crate::AppState;
use crate::dispatcher::Cmd;

#[get("/soc_current")]
async fn soc_current(data: web::Data<AppState>) -> impl Responder {
    let mut comms = data.comms.lock().unwrap();
    comms.tx_to_mygrid.send(Cmd::Soc).unwrap();

    if let Some(json) = comms.rx_from_mygrid.recv().await {
        HttpResponse::Ok().body(json)
    } else {
        HttpResponse::NoContent().finish()
    }
}

#[get("/soc_history")]
async fn soc_history(data: web::Data<AppState>) -> impl Responder {
    let mut comms = data.comms.lock().unwrap();
    comms.tx_to_mygrid.send(Cmd::SocHistory).unwrap();

    if let Some(json) = comms.rx_from_mygrid.recv().await {
        HttpResponse::Ok().body(json)
    } else {
        HttpResponse::NoContent().finish()
    }
}

#[get("/est_production")]
async fn est_production(data: web::Data<AppState>) -> impl Responder {
    let mut comms = data.comms.lock().unwrap();
    comms.tx_to_mygrid.send(Cmd::EstProduction).unwrap();

    if let Some(json) = comms.rx_from_mygrid.recv().await {
        HttpResponse::Ok().body(json)
    } else {
        HttpResponse::NoContent().finish()
    }
}

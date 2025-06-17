use actix_web::{get, web, HttpResponse, Responder};
use crate::AppState;
use crate::dispatcher::Cmd;

#[get("/small_dash_data")]
async fn small_dash_data(data: web::Data<AppState>) -> impl Responder {
    let mut comms = data.comms.lock().await;
    comms.tx_to_mygrid.send(Cmd::SmallDashData).unwrap();

    if let Some(json) = comms.rx_from_mygrid.recv().await {
        HttpResponse::Ok().body(json)
    } else {
        HttpResponse::NoContent().finish()
    }
}

#[get("/full_dash_data")]
async fn full_dash_data(data: web::Data<AppState>) -> impl Responder {
    let mut comms = data.comms.lock().await;
    comms.tx_to_mygrid.send(Cmd::FullDashData).unwrap();

    if let Some(json) = comms.rx_from_mygrid.recv().await {
        HttpResponse::Ok().body(json)
    } else {
        HttpResponse::NoContent().finish()
    }
}

#[get("/schedule")]
async fn schedule(data: web::Data<AppState>) -> impl Responder {
    let mut comms = data.comms.lock().await;
    comms.tx_to_mygrid.send(Cmd::Schedule).unwrap();

    if let Some(json) = comms.rx_from_mygrid.recv().await {
        HttpResponse::Ok().body(json)
    } else {
        HttpResponse::NoContent().finish()
    }
}

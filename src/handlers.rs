use actix_web::{get, web, HttpResponse, Responder};
use crate::AppState;
use crate::dispatcher::Cmd;

#[get("/combined_realtime")]
async fn combined_realtime(data: web::Data<AppState>) -> impl Responder {
    let mut comms = data.comms.lock().await;
    comms.tx_to_mygrid.send(Cmd::CombinedRealTime).unwrap();

    if let Some(json) = comms.rx_from_mygrid.recv().await {
        HttpResponse::Ok().body(json)
    } else {
        HttpResponse::NoContent().finish()
    }
}

#[get("/combined_production")]
async fn combined_production(data: web::Data<AppState>) -> impl Responder {
    let mut comms = data.comms.lock().await;
    comms.tx_to_mygrid.send(Cmd::CombinedProduction).unwrap();

    if let Some(json) = comms.rx_from_mygrid.recv().await {
        HttpResponse::Ok().body(json)
    } else {
        HttpResponse::NoContent().finish()
    }
}

#[get("/combined_load")]
async fn combined_load(data: web::Data<AppState>) -> impl Responder {
    let mut comms = data.comms.lock().await;
    comms.tx_to_mygrid.send(Cmd::CombinedLoad).unwrap();

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

#[get("/forecast_temp")]
async fn forecast_temp(data: web::Data<AppState>) -> impl Responder {
    let mut comms = data.comms.lock().await;
    comms.tx_to_mygrid.send(Cmd::ForecastTemp).unwrap();

    if let Some(json) = comms.rx_from_mygrid.recv().await {
        HttpResponse::Ok().body(json)
    } else {
        HttpResponse::NoContent().finish()
    }
}

#[get("/forecast_cloud")]
async fn forecast_cloud(data: web::Data<AppState>) -> impl Responder {
    let mut comms = data.comms.lock().await;
    comms.tx_to_mygrid.send(Cmd::ForecastCloud).unwrap();

    if let Some(json) = comms.rx_from_mygrid.recv().await {
        HttpResponse::Ok().body(json)
    } else {
        HttpResponse::NoContent().finish()
    }
}

#[get("/tariffs_buy")]
async fn tariffs_buy(data: web::Data<AppState>) -> impl Responder {
    let mut comms = data.comms.lock().await;
    comms.tx_to_mygrid.send(Cmd::TariffsBuy).unwrap();

    if let Some(json) = comms.rx_from_mygrid.recv().await {
        HttpResponse::Ok().body(json)
    } else {
        HttpResponse::NoContent().finish()
    }
}

#[get("/temperature")]
async fn temperature(data: web::Data<AppState>) -> impl Responder {
    let mut comms = data.comms.lock().await;
    comms.tx_to_mygrid.send(Cmd::Temperature).unwrap();

    if let Some(json) = comms.rx_from_mygrid.recv().await {
        HttpResponse::Ok().body(json)
    } else {
        HttpResponse::NoContent().finish()
    }
}

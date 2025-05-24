use actix_web::{get, web, HttpResponse, Responder};
use crate::AppState;
use crate::dispatcher::Cmd;

#[get("/soc")]
async fn soc(data: web::Data<AppState>) -> impl Responder {
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

#[get("/production")]
async fn production(data: web::Data<AppState>) -> impl Responder {
    let mut comms = data.comms.lock().unwrap();
    comms.tx_to_mygrid.send(Cmd::Production).unwrap();

    if let Some(json) = comms.rx_from_mygrid.recv().await {
        HttpResponse::Ok().body(json)
    } else {
        HttpResponse::NoContent().finish()
    }
}

#[get("/production_history")]
async fn production_history(data: web::Data<AppState>) -> impl Responder {
    let mut comms = data.comms.lock().unwrap();
    comms.tx_to_mygrid.send(Cmd::ProductionHistory).unwrap();

    if let Some(json) = comms.rx_from_mygrid.recv().await {
        HttpResponse::Ok().body(json)
    } else {
        HttpResponse::NoContent().finish()
    }
}

#[get("/load")]
async fn load(data: web::Data<AppState>) -> impl Responder {
    let mut comms = data.comms.lock().unwrap();
    comms.tx_to_mygrid.send(Cmd::Load).unwrap();

    if let Some(json) = comms.rx_from_mygrid.recv().await {
        HttpResponse::Ok().body(json)
    } else {
        HttpResponse::NoContent().finish()
    }
}

#[get("/load_history")]
async fn load_history(data: web::Data<AppState>) -> impl Responder {
    let mut comms = data.comms.lock().unwrap();
    comms.tx_to_mygrid.send(Cmd::LoadHistory).unwrap();

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

#[get("/est_load")]
async fn est_load(data: web::Data<AppState>) -> impl Responder {
    let mut comms = data.comms.lock().unwrap();
    comms.tx_to_mygrid.send(Cmd::EstLoad).unwrap();

    if let Some(json) = comms.rx_from_mygrid.recv().await {
        HttpResponse::Ok().body(json)
    } else {
        HttpResponse::NoContent().finish()
    }
}

#[get("/schedule")]
async fn schedule(data: web::Data<AppState>) -> impl Responder {
    let mut comms = data.comms.lock().unwrap();
    comms.tx_to_mygrid.send(Cmd::Schedule).unwrap();

    if let Some(json) = comms.rx_from_mygrid.recv().await {
        HttpResponse::Ok().body(json)
    } else {
        HttpResponse::NoContent().finish()
    }
}

#[get("/forecast")]
async fn forecast(data: web::Data<AppState>) -> impl Responder {
    let mut comms = data.comms.lock().unwrap();
    comms.tx_to_mygrid.send(Cmd::Forecast).unwrap();

    if let Some(json) = comms.rx_from_mygrid.recv().await {
        HttpResponse::Ok().body(json)
    } else {
        HttpResponse::NoContent().finish()
    }
}

#[get("/tariffs")]
async fn tariffs(data: web::Data<AppState>) -> impl Responder {
    let mut comms = data.comms.lock().unwrap();
    comms.tx_to_mygrid.send(Cmd::Tariffs).unwrap();

    if let Some(json) = comms.rx_from_mygrid.recv().await {
        HttpResponse::Ok().body(json)
    } else {
        HttpResponse::NoContent().finish()
    }
}

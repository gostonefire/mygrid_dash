use actix_web::{get, web, HttpResponse, Responder};
use crate::AppState;
use crate::dispatcher::Cmd;

#[get("/soc")]
async fn soc(data: web::Data<AppState>) -> impl Responder {
    let mut comms = data.comms.lock().await;
    comms.tx_to_mygrid.send(Cmd::Soc).unwrap();

    if let Some(json) = comms.rx_from_mygrid.recv().await {
        HttpResponse::Ok().body(json)
    } else {
        HttpResponse::NoContent().finish()
    }
}

#[get("/soc_history")]
async fn soc_history(data: web::Data<AppState>) -> impl Responder {
    let mut comms = data.comms.lock().await;
    comms.tx_to_mygrid.send(Cmd::SocHistory).unwrap();

    if let Some(json) = comms.rx_from_mygrid.recv().await {
        HttpResponse::Ok().body(json)
    } else {
        HttpResponse::NoContent().finish()
    }
}

#[get("/production")]
async fn production(data: web::Data<AppState>) -> impl Responder {
    let mut comms = data.comms.lock().await;
    comms.tx_to_mygrid.send(Cmd::Production).unwrap();

    if let Some(json) = comms.rx_from_mygrid.recv().await {
        HttpResponse::Ok().body(json)
    } else {
        HttpResponse::NoContent().finish()
    }
}

#[get("/production_history")]
async fn production_history(data: web::Data<AppState>) -> impl Responder {
    let mut comms = data.comms.lock().await;
    comms.tx_to_mygrid.send(Cmd::ProductionHistory).unwrap();

    if let Some(json) = comms.rx_from_mygrid.recv().await {
        HttpResponse::Ok().body(json)
    } else {
        HttpResponse::NoContent().finish()
    }
}

#[get("/load")]
async fn load(data: web::Data<AppState>) -> impl Responder {
    let mut comms = data.comms.lock().await;
    comms.tx_to_mygrid.send(Cmd::Load).unwrap();

    if let Some(json) = comms.rx_from_mygrid.recv().await {
        HttpResponse::Ok().body(json)
    } else {
        HttpResponse::NoContent().finish()
    }
}

#[get("/load_history")]
async fn load_history(data: web::Data<AppState>) -> impl Responder {
    let mut comms = data.comms.lock().await;
    comms.tx_to_mygrid.send(Cmd::LoadHistory).unwrap();

    if let Some(json) = comms.rx_from_mygrid.recv().await {
        HttpResponse::Ok().body(json)
    } else {
        HttpResponse::NoContent().finish()
    }
}

#[get("/est_production")]
async fn est_production(data: web::Data<AppState>) -> impl Responder {
    println!("est_production");
    let mut comms = data.comms.lock().await;
    comms.tx_to_mygrid.send(Cmd::EstProduction).unwrap();
    println!("est_production");

    if let Some(json) = comms.rx_from_mygrid.recv().await {
        HttpResponse::Ok().body(json)
    } else {
        HttpResponse::NoContent().finish()
    }
}

#[get("/est_load")]
async fn est_load(data: web::Data<AppState>) -> impl Responder {
    let mut comms = data.comms.lock().await;
    comms.tx_to_mygrid.send(Cmd::EstLoad).unwrap();
    
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

#[get("/tariffs_sell")]
async fn tariffs_sell(data: web::Data<AppState>) -> impl Responder {
    let mut comms = data.comms.lock().await;
    comms.tx_to_mygrid.send(Cmd::TariffsSell).unwrap();

    if let Some(json) = comms.rx_from_mygrid.recv().await {
        HttpResponse::Ok().body(json)
    } else {
        HttpResponse::NoContent().finish()
    }
}

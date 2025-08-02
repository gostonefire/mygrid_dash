use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use actix_web::cookie::{Cookie, SameSite};
use actix_web::http::header::LOCATION;
use chrono::Utc;
use log::{error, info};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::AppState;
use crate::dispatcher::Cmd;
use crate::manager_tokens::{build_access_request_url, Tokens};

const X_REDIRECT: &str = "X-Redirect-Location";
const SESSION_COOKIE: &str = "mygrid_dash";

#[derive(Deserialize, Serialize)]
struct State {
    session: String,
    state_code: String,
    context: String,
}
#[derive(Deserialize)]
struct Params {
    state: String,
    code: String,
}

#[derive(Deserialize)]
struct Context {
    context: String,
}

#[get("/data/{dash_type}")]
async fn get_data(data: web::Data<AppState>, path: web::Path<String>, req: HttpRequest) -> impl Responder {
    let cmd: Cmd;
    let redirect: &str;
    let dash_type = path.into_inner();
    
    if dash_type == "small" {
        cmd = Cmd::SmallDashData;
        redirect = "/login?context=/";
    } else if dash_type == "full" {
        cmd = Cmd::FullDashData;
        redirect = "/login?context=/full";
    } else {
        return HttpResponse::BadRequest().finish();
    }
    
    if let Some(cookie) = req.cookie(SESSION_COOKIE) {
        if let Some((_, _, tokens )) = data.sessions.read().await.get(&cookie.value().to_string()) {
            if tokens.as_ref().is_some_and(|t| !t.is_expired()) {
                let mut comms = data.comms.lock().await;
                comms.tx_to_mygrid.send(cmd).unwrap();

                return if let Some(json) = comms.rx_from_mygrid.recv().await {
                    HttpResponse::Ok().body(json)
                } else {
                    HttpResponse::NoContent().finish()
                }
            }
        }
    }

    HttpResponse::Ok()
        .append_header((X_REDIRECT, redirect))
        .body("{\"message\": \"redirect\"}")
}

#[get("/login")]
async fn login(data: web::Data<AppState>, context: web::Query<Context>) -> impl Responder {
    let session = Uuid::new_v4().to_string();
    let state_code = Uuid::new_v4().to_string();

    match serde_json::to_string(&State { session: session.clone(), state_code: state_code.clone(), context: context.context.clone() }) {
        Ok(state) => {
            data.sessions.write().await.insert(session, (Utc::now().timestamp(), state_code, None));

            let url = build_access_request_url(&data.config, &state).await;

            HttpResponse::TemporaryRedirect()
                .append_header((LOCATION, url))
                .finish()
        }
        Err(e) => {
            error!("error in /login: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[get("/code")]
async fn code(data: web::Data<AppState>, params: web::Query<Params>) -> impl Responder {
    if let Ok(state) = serde_json::from_str::<State>(&params.state) {
        let mut sessions = data.sessions.write().await;

        if let Some((_, sc, _)) = sessions.get(&state.session) {
            if &state.state_code == sc {
                return match Tokens::from_code(&data.config, &params.code).await {
                    Ok(token) => {
                        info!("{} tries to login", token.email);
                        if token.is_authorized() {
                            sessions.insert(state.session.clone(), (Utc::now().timestamp(), String::new(), Some(token)));

                            let cookie = Cookie::build(SESSION_COOKIE, state.session)
                                .expires(None)
                                .secure(true)
                                .http_only(true)
                                .same_site(SameSite::Lax)
                                .finish();

                            HttpResponse::SeeOther()
                                .cookie(cookie)
                                .append_header((LOCATION, state.context))
                                .finish()
                        } else {
                            HttpResponse::SeeOther()
                                .append_header((LOCATION, "/unauthorized.html"))
                                .finish()
                        }
                    }
                    Err(e) => {
                        error!("error in /code: {}", e);
                        HttpResponse::InternalServerError().finish()
                    }
                }

            }
        }
    }

    HttpResponse::SeeOther()
        .append_header((LOCATION, "/unauthorized.html"))
        .finish()
}

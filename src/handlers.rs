use axum::extract::{Path, Query, State};
use axum::http::{header, HeaderName, StatusCode};
use axum::response::{IntoResponse, Redirect};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use chrono::Utc;
use tracing::{error, info};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::AppState;
use crate::dispatcher::Cmd;
use crate::manager_tokens::{build_access_request_url, Tokens};

const X_REDIRECT: HeaderName = HeaderName::from_static("x-redirect-location");
const SESSION_COOKIE: &str = "mygrid_dash";

#[derive(Deserialize, Serialize)]
struct AuthState {
    session: String,
    state_code: String,
    context: String,
}
#[derive(Deserialize)]
pub struct Params {
    state: String,
    code: String,
}

#[derive(Deserialize)]
pub struct Context {
    context: String,
}

//async fn get_data(data: web::Data<AppState>, path: web::Path<String>, req: HttpRequest) -> impl Responder {
pub async fn get_data(Path(dash_type): Path<String>, State(data): State<AppState>, jar: CookieJar) -> impl IntoResponse {
    let cmd: Cmd;
    let redirect: &str;

    if dash_type == "small" {
        cmd = Cmd::SmallDashData;
        redirect = "/login?context=/";
    } else if dash_type == "full" {
        cmd = Cmd::FullDashData;
        redirect = "/login?context=/full";
    } else {
        return StatusCode::BAD_REQUEST.into_response();
    }
    
    if let Some(cookie) = jar.get(SESSION_COOKIE) {
        if let Some((_, _, tokens )) = data.sessions.read().await.get(&cookie.value().to_string()) {
            if tokens.as_ref().is_some_and(|t| !t.is_expired()) {
                let mut comms = data.comms.lock().await;
                comms.tx_to_mygrid.send(cmd).unwrap();

                return if let Some(json) = comms.rx_from_mygrid.recv().await {
                    ([(header::CONTENT_TYPE, "application/json")], json).into_response()
                } else {
                    StatusCode::NO_CONTENT.into_response()
                }
            }
        }
    }

    ([(header::CONTENT_TYPE, "application/json"), (X_REDIRECT, redirect)], "{\"message\": \"redirect\"}").into_response()
}

pub async fn login(State(data): State<AppState>, Query(context): Query<Context>) -> impl IntoResponse {
    let session = Uuid::new_v4().to_string();
    let state_code = Uuid::new_v4().to_string();

    match serde_json::to_string(&AuthState { session: session.clone(), state_code: state_code.clone(), context: context.context.clone() }) {
        Ok(state) => {
            data.sessions.write().await.insert(session, (Utc::now().timestamp(), state_code, None));

            let url = build_access_request_url(&data.config, &state).await;

            Redirect::temporary(&url).into_response()
        }
        Err(e) => {
            error!("error in /login: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn code(State(data): State<AppState>, Query(params): Query<Params>, jar: CookieJar) -> impl IntoResponse {
    if let Ok(state) = serde_json::from_str::<AuthState>(&params.state) {
        let mut sessions = data.sessions.write().await;

        if let Some((_, sc, _)) = sessions.get(&state.session) {
            if &state.state_code == sc {
                return match Tokens::from_code(&data.config, &params.code).await {
                    Ok(token) => {
                        info!("{} tries to login", token.email);
                        if token.is_authorized() {
                            sessions.insert(state.session.clone(), (Utc::now().timestamp(), String::new(), Some(token)));

                            let cookie = Cookie::build((SESSION_COOKIE, state.session))
                                .expires(None)
                                .secure(true)
                                .http_only(true)
                                .same_site(SameSite::Lax);

                            let jar = jar.add(cookie);

                            (jar, Redirect::to(&state.context)).into_response()
                        } else {
                            Redirect::to("/unauthorized.html").into_response()
                        }
                    }
                    Err(e) => {
                        error!("error in /code: {}", e);
                        StatusCode::INTERNAL_SERVER_ERROR.into_response()
                    }
                }

            }
        }
    }

    Redirect::to("/unauthorized.html").into_response()
}

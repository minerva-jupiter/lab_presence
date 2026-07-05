mod auth;
use auth::{AuthenticatedDevice, generate_secure_token, validate_device};

use actix_web::{
    App, HttpRequest, HttpResponse, HttpServer, Responder, get, http::header::ContentType,
    middleware, post, put, web,
};
use log::debug;
use rcgen::generate_simple_self_signed;
use rustls::{
    ServerConfig,
    pki_types::{CertificateDer, PrivateKeyDer},
};
use shared::*;
use std::collections::HashMap;
use std::sync::Mutex;

struct AppState {
    // userId -> PresenceResponse
    presences: Mutex<HashMap<String, PresenceResponse>>,
    // token -> userId
    tokens: Mutex<HashMap<String, String>>,
}

/// simple handle
async fn index(req: HttpRequest) -> HttpResponse {
    debug!("{req:?}");

    HttpResponse::Ok().content_type(ContentType::html()).body(
        "<!DOCTYPE html><html><body>\
            <p>Welcome to your TLS-secured homepage!</p>\
        </body></html>",
    )
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let config = load_rustls_config();

    let state = web::Data::new(AppState {
        presences: Mutex::new(HashMap::new()),
        tokens: Mutex::new(HashMap::new()),
    });

    log::info!("starting HTTPS server at https://localhost:8443");

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .wrap(middleware::Logger::default())
            .service(login)
            .service(get_presence)
            .service(put_presence)
            .service(web::resource("/index.html").to(index))
            .service(web::redirect("/", "/index.html"))
    })
    .bind_rustls_0_23("127.0.0.1:8443", config)?
    .run()
    .await
}

fn load_rustls_config() -> rustls::ServerConfig {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .unwrap();

    let subject_alt_names = vec!["localhost".to_string(), "127.0.0.1".to_string()];
    let cert = generate_simple_self_signed(subject_alt_names).unwrap();

    let cert_chain = vec![CertificateDer::from(cert.cert.der().to_vec())];
    let key_der = PrivateKeyDer::Pkcs8(cert.signing_key.serialize_der().into());

    ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert_chain, key_der)
        .unwrap()
}

#[post("/api/v1/login")]
async fn login(data: web::Json<LoginRequest>, state: web::Data<AppState>) -> impl Responder {
    if !validate_device(&data.device_id, &data.device_secret) {
        return HttpResponse::Forbidden().json(ErrorResponse {
            errcode: "M_FORBIDDEN".to_string(),
            error: "authorization failed".to_string(),
        });
    }

    let token = generate_secure_token();
    let mut tokens = state.tokens.lock().unwrap();
    tokens.insert(token.clone(), data.device_id.clone());

    HttpResponse::Ok().json(LoginResponse {
        access_token: token,
        expires_in: 3600,
    })
}

#[get("/api/v1/presence/{userId}/status")]
async fn get_presence(
    path: web::Path<String>,
    _auth: AuthenticatedDevice,
    state: web::Data<AppState>,
) -> impl Responder {
    let user_id = path.into_inner();
    let presences = state.presences.lock().unwrap();

    if let Some(presence) = presences.get(&user_id) {
        HttpResponse::Ok().json(presence)
    } else {
        HttpResponse::NotFound().json(ErrorResponse {
            errcode: "M_UNKNOWN".to_string(),
            error: "An unknown error occurred".to_string(),
        })
    }
}

#[put("/api/v1/presence/{userId}/status")]
async fn put_presence(
    path: web::Path<String>,
    data: web::Json<PresenceUpdateRequest>,
    _auth: AuthenticatedDevice,
    state: web::Data<AppState>,
) -> impl Responder {
    let user_id = path.into_inner();
    let mut presences = state.presences.lock().unwrap();

    presences.insert(
        user_id,
        PresenceResponse {
            presence: data.presence.clone(),
            status_msg: data.status_msg.clone(),
            timestamp: data.timestamp,
        },
    );

    HttpResponse::Ok().json(serde_json::json!({}))
}

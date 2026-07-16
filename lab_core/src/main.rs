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
use std::sync::Mutex;
use std::{collections::HashMap, time::UNIX_EPOCH};

struct AppState {
    presences: Mutex<HashMap<String, LabPresence>>,
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
        HttpResponse::Ok().json(PresenceResponse {
            presence: presence.presence().to_string(),
            status_msg: presence.status_msg().unwrap_or("").to_string(),
            timestamp: presence.timestamp(),
        })
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

    let status_msg = if data.status_msg.is_empty() {
        None
    } else {
        Some(data.status_msg.clone())
    };

    if let Some(existing) = presences.get_mut(&user_id) {
        match existing.update(
            data.presence.clone(),
            status_msg,
            std::time::SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as u64,
        ) {
            Ok(_) => HttpResponse::Ok().json(serde_json::json!({})),
            Err(e) => HttpResponse::BadRequest().json(ErrorResponse {
                errcode: "M_BAD_JSON".to_string(),
                error: e.to_string(),
            }),
        }
    } else {
        presences.insert(
            user_id,
            LabPresence::new(
                data.presence.clone(),
                status_msg,
                std::time::SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as u64,
            ),
        );
        HttpResponse::Ok().json(serde_json::json!({}))
    }
}

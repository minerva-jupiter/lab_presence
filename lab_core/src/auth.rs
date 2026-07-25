use actix_web::{
    FromRequest, HttpRequest, dev::Payload, error::ResponseError, http::header::AUTHORIZATION,
};
use rand::RngExt;
use rand::distr::Alphanumeric;
use serde_json::Value;
use shared::ErrorResponse;
use std::env;
use std::future::{Ready, ready};

pub struct AuthenticatedDevice {
    #[allow(dead_code)]
    pub device_id: String,
}

#[derive(Debug)]
pub enum AuthError {
    MissingHeader,
    InvalidFormat,
    Unauthorized,
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::MissingHeader => write!(f, "Missing Authorization Header"),
            AuthError::InvalidFormat => write!(f, "Invalid Bearer Format"),
            AuthError::Unauthorized => write!(f, "Authorization failed"),
        }
    }
}

impl std::error::Error for AuthError {}

impl ResponseError for AuthError {
    fn error_response(&self) -> actix_web::HttpResponse {
        actix_web::HttpResponse::Forbidden().json(ErrorResponse {
            errcode: "M_FORBIDDEN".to_string(),
            error: self.to_string(),
        })
    }
}

impl FromRequest for AuthenticatedDevice {
    type Error = AuthError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let auth_header = match req.headers().get(AUTHORIZATION) {
            Some(h) => h,
            None => return ready(Err(AuthError::MissingHeader)),
        };

        let auth_str = match auth_header.to_str() {
            Ok(s) => s,
            Err(_) => return ready(Err(AuthError::InvalidFormat)),
        };

        if !auth_str.starts_with("Bearer ") {
            return ready(Err(AuthError::InvalidFormat));
        }

        let token = auth_str.trim_start_matches("Bearer ");

        let state = match req.app_data::<actix_web::web::Data<crate::AppState>>() {
            Some(s) => s,
            None => return ready(Err(AuthError::Unauthorized)),
        };

        let tokens = state.tokens.lock().unwrap();

        if let Some(device_id) = tokens.get(token) {
            ready(Ok(AuthenticatedDevice {
                device_id: device_id.clone(),
            }))
        } else {
            ready(Err(AuthError::Unauthorized))
        }
    }
}

pub fn validate_device(device_id: &str, secret: &str) -> bool {
    let _ = dotenv::dotenv();
    let devices_raw = env::var("ALLOWED_DEVICES").unwrap_or_else(|_| "{}".to_string());

    if let Ok(Value::Object(map)) = serde_json::from_str(&devices_raw) {
        if let Some(Value::String(stored_secret)) = map.get(device_id) {
            return stored_secret == secret;
        }
    }
    false
}

pub fn generate_secure_token() -> String {
    use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};

    const LENGTH: usize = 32;

    let token: String = rand::rng()
        .sample_iter(&Alphanumeric)
        .take(LENGTH)
        .map(char::from)
        .collect();
    URL_SAFE_NO_PAD.encode(token)
}

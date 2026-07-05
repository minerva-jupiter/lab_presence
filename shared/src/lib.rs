use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::error::Error;

pub struct LabPresence {
    presence: String,
    status_msg: Option<String>,
    timestamp: u64,
}

impl LabPresence {
    pub fn new(presence: String, status_msg: Option<String>, timestamp: u64) -> Self {
        Self {
            presence,
            status_msg,
            timestamp,
        }
    }

    pub fn presence(&self) -> &str {
        &self.presence
    }

    pub fn status_msg(&self) -> Option<&str> {
        self.status_msg.as_deref()
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn set(
        &mut self,
        presence: String,
        status_msg: Option<String>,
        timestamp: u64,
    ) -> Result<(), Box<dyn Error>> {
        self.presence = presence;
        self.status_msg = status_msg;
        self.timestamp = timestamp;
        Ok(())
    }

    pub fn update(
        &mut self,
        presence: String,
        status_msg: Option<String>,
        timestamp: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if timestamp > self.timestamp {
            self.presence = presence;
            self.status_msg = status_msg;
            self.timestamp = timestamp;
            Ok(())
        } else {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "timestamp is not greater than current timestamp",
            )))
        }
    }
}

pub struct UserId {
    pub id: String,
}
impl UserId {
    pub fn get_id(idm: String) -> Result<Self, Box<dyn std::error::Error>> {
        let _ = dotenv();
        let salt = std::env::var("SALT")?;
        let mut hasher = sha2::Sha256::new();
        hasher.update(idm.as_bytes());
        hasher.update(salt.as_bytes());
        let result = hasher
            .finalize()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();

        Ok(Self { id: result })
    }
}
#[derive(Deserialize)]
pub struct LoginRequest {
    pub device_id: String,
    pub device_secret: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub expires_in: u32,
}

#[derive(Serialize)]
pub struct PresenceResponse {
    pub presence: String,
    pub status_msg: String,
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize)]
pub struct PresenceUpdateRequest {
    pub presence: String,
    pub status_msg: String,
    pub timestamp: u64,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub errcode: String,
    pub error: String,
}

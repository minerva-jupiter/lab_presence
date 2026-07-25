use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Serialize, Deserialize, Clone)]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserId {
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub id: UserId,
    pub card_serial_number: String,
}

#[derive(Serialize, Deserialize)]
pub struct MemberDirectory {
    pub users: Vec<User>,
}
impl MemberDirectory {
    pub fn new(users: Vec<User>) -> Self {
        Self { users }
    }
    pub fn init_from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let member_directory = std::env::var("MEMBER_DIRECTORY")?;
        let users: Vec<User> = serde_json::from_str(&member_directory)?;
        Ok(Self::new(users))
    }
    pub fn inquire(&self, card_serial_number: &str) -> Result<UserId, Box<dyn std::error::Error>> {
        self.users
            .iter()
            .find(|u| u.card_serial_number == card_serial_number)
            .map(|u| u.id.clone())
            .ok_or_else(|| "User not found".into())
    }
}

#[derive(Serialize, Deserialize)]
pub struct LoginRequest {
    pub device_id: String,
    pub device_secret: String,
}

#[derive(Serialize, Deserialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub expires_in: u32,
}

#[derive(Serialize, Deserialize)]
pub struct PresenceResponse {
    pub presence: String,
    pub status_msg: String,
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize)]
pub struct PresenceUpdateRequest {
    pub presence: String,
    pub status_msg: String,
}

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub errcode: String,
    pub error: String,
}

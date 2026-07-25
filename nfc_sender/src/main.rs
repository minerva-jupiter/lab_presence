use pcsc::{Context, Protocols, Scope, ShareMode};
use shared::{MemberDirectory, UserId};
use std::thread::sleep;
use std::time::Duration;

struct NfcSender {
    core_url: String,
    token: String,
    member_directory: MemberDirectory,
}

impl NfcSender {
    fn new() -> Self {
        Self {
            core_url: String::new(),
            token: String::new(),
            member_directory: MemberDirectory::new(vec![]),
        }
    }
    fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.core_url =
            std::env::var("CORE_URL").unwrap_or_else(|_| "https://localhost:8443".to_string());
        self.member_directory = serde_json::from_str(&std::env::var("MEMBER_DIRECTORY")?)?;
        let allowed_devices: serde_json::Value =
            serde_json::from_str(&std::env::var("ALLOWED_DEVICES")?)?;
        let secret: &str = allowed_devices["nfc"]
            .as_str()
            .ok_or("ALLOWED_DEVICES.nfc is not a string or missing")?;
        self.login(secret.to_string())?;
        Ok(())
    }

    fn login(&mut self, secret: String) -> Result<(), Box<dyn std::error::Error>> {
        let login_request = shared::LoginRequest {
            device_id: "nfc".to_string(),
            device_secret: secret,
        };
        let response: shared::LoginResponse =
            ureq::post(&format!("{}/api/v1/login", self.core_url))
                .send_json(login_request)?
                .body_mut()
                .read_json::<shared::LoginResponse>()?;
        self.token = response.access_token;
        Ok(())
    }
    fn update_presence(&self, user_id: &UserId, presence: &str) -> Result<(), ureq::Error> {
        let request_body = shared::PresenceUpdateRequest {
            presence: presence.to_string(),
            status_msg: "NFC toggle request sent".to_string(),
        };
        let _ = ureq::put(&format!(
            "{}/api/v1/presence/{}/status",
            self.core_url, user_id.id
        ))
        .header("Authorization", &format!("Bearer {}", self.token))
        .send_json(request_body)?;
        Ok(())
    }
    fn get_presence(&self, user_id: &UserId) -> Result<String, ureq::Error> {
        let now_presence_res = ureq::get(&format!(
            "{}/api/v1/presence/{}/status",
            self.core_url, user_id.id
        ))
        .header("Authorization", &format!("Bearer {}", self.token))
        .call()?
        .body_mut()
        .read_json::<shared::PresenceResponse>()?;
        Ok(now_presence_res.presence)
    }
    fn toggled_presence(presence: &str) -> &'static str {
        if presence == "inLab" {
            "offline"
        } else {
            "inLab"
        }
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = dotenvy::dotenv();
    let ctx = Context::establish(Scope::User)?;

    let mut nfc_sender: NfcSender = NfcSender::new();
    nfc_sender.init()?;

    println!("NFC sender has started...");

    loop {
        let mut readers_buf = [0u8; 2048];
        let mut readers = match ctx.list_readers(&mut readers_buf) {
            Ok(r) => r,
            Err(_) => {
                sleep(Duration::from_secs(1));
                continue;
            }
        };

        let reader_name = match readers.next() {
            Some(name) => name,
            None => {
                sleep(Duration::from_secs(1));
                continue;
            }
        };

        if let Ok(card) = ctx.connect(reader_name, ShareMode::Shared, Protocols::ANY) {
            let felica_poll_cmd = [
                0xFF, 0x00, 0x00, 0x00, 0x08, 0xD4, 0x40, 0x01, 0x00, 0xFF, 0xFF, 0x01, 0x00,
            ];
            let mut rapdu_buf = [0u8; 258];

            if let Ok(rapdu) = card.transmit(&felica_poll_cmd, &mut rapdu_buf) {
                if rapdu.len() >= 18 {
                    let idm_bytes = &rapdu[10..18];
                    let idm_str = idm_bytes
                        .iter()
                        .map(|b| format!("{:02X}", b))
                        .collect::<String>();

                    if let Ok(user_id) = nfc_sender.member_directory.inquire(&idm_str) {
                        if let Ok(current_presence) = nfc_sender.get_presence(&user_id) {
                            let toggled = NfcSender::toggled_presence(&current_presence);
                            let _ = nfc_sender.update_presence(&user_id, toggled);
                        }
                    }

                    sleep(Duration::from_secs(2));
                }
            }
        }

        sleep(Duration::from_millis(500));
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    fn extract_idm(rapdu: &[u8]) -> Option<String> {
        if rapdu.len() < 18 {
            return None;
        }
        Some(
            rapdu[10..18]
                .iter()
                .map(|b| format!("{:02X}", b))
                .collect::<String>(),
        )
    }

    #[test]
    fn test_toggled_presence_in_lab_to_offline() {
        let current = "inLab";
        let next = NfcSender::toggled_presence(current);
        assert_eq!(next, "offline");
    }

    #[test]
    fn test_toggled_presence_offline_to_in_lab() {
        let current = "offline";
        let next = NfcSender::toggled_presence(current);
        assert_eq!(next, "inLab");
    }

    #[test]
    fn test_toggled_presence_unknown_status_defaults_to_in_lab() {
        let current = "unknown";
        let next = NfcSender::toggled_presence(current);
        assert_eq!(next, "inLab");
    }

    #[test]
    fn test_extract_idm_valid_rapdu() {
        let mut rapdu = vec![0u8; 18];
        rapdu[10..18].copy_from_slice(&[0x01, 0x2E, 0x3F, 0x4A, 0x5B, 0x6C, 0x7D, 0x8E]);

        let idm = extract_idm(&rapdu);
        assert_eq!(idm, Some("012E3F4A5B6C7D8E".to_string()));
    }

    #[test]
    fn test_extract_idm_short_rapdu_returns_none() {
        let rapdu = vec![0u8; 17];
        let idm = extract_idm(&rapdu);
        assert_eq!(idm, None);
    }
}
#[cfg(test)]
mod integration_tests {
    use super::*;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_nfc_sender_api_flow() -> Result<(), Box<dyn std::error::Error>> {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/v1/presence/user_123/status"))
            .and(header("Authorization", "Bearer mock_token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "presence": "inLab",
                "status_msg": "NFC toggle request sent",
                "timestamp": 1625097600
            })))
            .mount(&mock_server)
            .await;

        Mock::given(method("PUT"))
            .and(path("/api/v1/presence/user_123/status"))
            .and(header("Authorization", "Bearer mock_token"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let sender = NfcSender {
            core_url: mock_server.uri(),
            token: "mock_token".to_string(),
            member_directory: MemberDirectory::new(vec![]),
        };

        let dummy_user = UserId {
            id: "user_123".to_string(),
        };

        let presence = sender.get_presence(&dummy_user)?;
        assert_eq!(presence, "inLab");

        let toggled = NfcSender::toggled_presence(&presence);
        let update_res = sender.update_presence(&dummy_user, toggled);
        assert!(update_res.is_ok());

        Ok(())
    }
}

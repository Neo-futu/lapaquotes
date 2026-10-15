//! WhatsApp Cloud API: send text messages.

use serde_json::json;
use tracing::{info, warn};

#[derive(Debug, Clone)]
pub struct WhatsAppConfig {
    pub access_token: String,
    pub phone_number_id: String,
    pub verify_token: String,
    pub api_version: String,
    pub api_base: String,
    /// If non-empty, only these WhatsApp IDs (digits) may chat.
    pub allowed_from: Vec<String>,
}

impl WhatsAppConfig {
    pub fn from_env() -> Option<Self> {
        let access_token = std::env::var("WHATSAPP_ACCESS_TOKEN")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())?;
        let phone_number_id = std::env::var("WHATSAPP_PHONE_NUMBER_ID")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())?;
        let verify_token = std::env::var("WHATSAPP_VERIFY_TOKEN")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())?;
        let api_version = std::env::var("WHATSAPP_API_VERSION")
            .unwrap_or_else(|_| "v21.0".into())
            .trim()
            .trim_start_matches('/')
            .to_string();
        let api_base = std::env::var("WHATSAPP_API_BASE")
            .unwrap_or_else(|_| "https://graph.facebook.com".into())
            .trim_end_matches('/')
            .to_string();
        let allowed_from = std::env::var("WHATSAPP_ALLOWED_FROM")
            .ok()
            .map(|s| {
                s.split(',')
                    .map(|p| p.trim().trim_start_matches('+').to_string())
                    .filter(|p| !p.is_empty())
                    .collect()
            })
            .unwrap_or_default();
        Some(Self {
            access_token,
            phone_number_id,
            verify_token,
            api_version,
            api_base,
            allowed_from,
        })
    }

    pub fn allows(&self, from: &str) -> bool {
        if self.allowed_from.is_empty() {
            return true;
        }
        let from = from.trim_start_matches('+');
        self.allowed_from.iter().any(|a| a == from)
    }
}

/// WhatsApp text body hard limit is 4096 characters.
fn truncate_wa(s: &str) -> String {
    const MAX: usize = 4000;
    let mut it = s.chars();
    let out: String = it.by_ref().take(MAX).collect();
    if it.next().is_some() {
        format!("{out}\n…(已截断)")
    } else {
        out
    }
}

pub async fn send_text(cfg: &WhatsAppConfig, to: &str, body: &str) -> Result<(), String> {
    let to = to.trim_start_matches('+');
    let url = format!(
        "{}/{}/{}/messages",
        cfg.api_base, cfg.api_version, cfg.phone_number_id
    );
    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .bearer_auth(&cfg.access_token)
        .json(&json!({
            "messaging_product": "whatsapp",
            "recipient_type": "individual",
            "to": to,
            "type": "text",
            "text": {
                "preview_url": false,
                "body": truncate_wa(body),
            }
        }))
        .send()
        .await
        .map_err(|e| format!("whatsapp send request: {e}"))?;
    let status = resp.status();
    let text = resp
        .text()
        .await
        .map_err(|e| format!("whatsapp send body: {e}"))?;
    if !status.is_success() {
        warn!(%text, %status, "whatsapp send failed");
        return Err(format!("whatsapp send HTTP {status}: {text}"));
    }
    info!(to, "whatsapp message sent");
    Ok(())
}

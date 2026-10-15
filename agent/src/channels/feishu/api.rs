//! Feishu Open API helpers (tenant token + message reply).

use serde::Deserialize;
use serde_json::json;
use tracing::{info, warn};

#[derive(Debug, Clone)]
pub struct FeishuConfig {
    pub app_id: String,
    pub app_secret: String,
    pub verification_token: Option<String>,
    pub api_base: String,
}

impl FeishuConfig {
    pub fn from_env() -> Option<Self> {
        let app_id = std::env::var("FEISHU_APP_ID")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())?;
        let app_secret = std::env::var("FEISHU_APP_SECRET")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())?;
        let verification_token = std::env::var("FEISHU_VERIFICATION_TOKEN")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        let api_base = std::env::var("FEISHU_API_BASE")
            .unwrap_or_else(|_| "https://open.feishu.cn".into())
            .trim_end_matches('/')
            .to_string();
        Some(Self {
            app_id,
            app_secret,
            verification_token,
            api_base,
        })
    }

    pub fn is_configured() -> bool {
        Self::from_env().is_some()
    }
}

#[derive(Debug, Deserialize)]
struct TokenResp {
    code: i64,
    #[serde(default)]
    msg: String,
    #[serde(default)]
    tenant_access_token: Option<String>,
    #[serde(default)]
    expire: Option<i64>,
}

pub async fn tenant_access_token(cfg: &FeishuConfig) -> Result<String, String> {
    let client = reqwest::Client::new();
    let url = format!("{}/open-apis/auth/v3/tenant_access_token/internal", cfg.api_base);
    let resp = client
        .post(&url)
        .json(&json!({
            "app_id": cfg.app_id,
            "app_secret": cfg.app_secret,
        }))
        .send()
        .await
        .map_err(|e| format!("feishu token request: {e}"))?;
    let body: TokenResp = resp
        .json()
        .await
        .map_err(|e| format!("feishu token parse: {e}"))?;
    if body.code != 0 {
        return Err(format!("feishu token code={}: {}", body.code, body.msg));
    }
    let token = body
        .tenant_access_token
        .ok_or_else(|| "feishu token missing tenant_access_token".to_string())?;
    info!(expire = ?body.expire, "feishu tenant_access_token ok");
    Ok(token)
}

pub async fn reply_text(
    cfg: &FeishuConfig,
    message_id: &str,
    text: &str,
) -> Result<(), String> {
    let token = tenant_access_token(cfg).await?;
    let client = reqwest::Client::new();
    let url = format!(
        "{}/open-apis/im/v1/messages/{}/reply",
        cfg.api_base, message_id
    );
    // content must be a JSON string per Feishu API
    let content = serde_json::to_string(&json!({ "text": text }))
        .map_err(|e| e.to_string())?;
    let resp = client
        .post(&url)
        .bearer_auth(&token)
        .json(&json!({
            "msg_type": "text",
            "content": content,
        }))
        .send()
        .await
        .map_err(|e| format!("feishu reply request: {e}"))?;
    let status = resp.status();
    let body = resp
        .text()
        .await
        .map_err(|e| format!("feishu reply body: {e}"))?;
    if !status.is_success() {
        return Err(format!("feishu reply HTTP {status}: {body}"));
    }
    let v: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| format!("feishu reply json: {e}; {body}"))?;
    let code = v.get("code").and_then(|c| c.as_i64()).unwrap_or(-1);
    if code != 0 {
        warn!(%body, "feishu reply non-zero code");
        return Err(format!("feishu reply code={code}: {body}"));
    }
    Ok(())
}

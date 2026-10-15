//! Feishu event webhook: URL verification + receive message → AgentService.

use axum::{
    Json, Router,
    extract::{FromRef, State},
    http::StatusCode,
    response::IntoResponse,
    routing::post,
};
use cornucopia::deadpool_postgres::Pool;
use serde::Deserialize;
use serde_json::{Value, json};
use tracing::{error, info, warn};

use super::api::{FeishuConfig, reply_text};
use crate::{
    api::{ChatRequest, ChatTurn},
    service::AgentService,
};

pub fn feishu_router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    Pool: FromRef<S>,
{
    Router::new().route("/events", post(handle_event::<S>))
}

#[derive(Debug, Deserialize)]
struct ChallengeBody {
    challenge: Option<String>,
    token: Option<String>,
    #[serde(rename = "type")]
    #[allow(dead_code)]
    typ: Option<String>,
    #[allow(dead_code)]
    encrypt: Option<String>,
}

pub async fn handle_event<S>(
    State(state): State<S>,
    Json(body): Json<Value>,
) -> impl IntoResponse
where
    S: Clone + Send + Sync + 'static,
    Pool: FromRef<S>,
{
    // Encrypted payloads need FEISHU_ENCRYPT_KEY decrypt — not implemented yet.
    if body.get("encrypt").and_then(|v| v.as_str()).is_some() {
        warn!("feishu encrypted event received; disable Encrypt Key or add decrypt support");
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "encrypted events not supported yet; clear Encrypt Key in Feishu console"
            })),
        )
            .into_response();
    }

    // URL verification (must return challenge within 1s)
    let is_challenge = body.get("type").and_then(|v| v.as_str()) == Some("url_verification")
        || (body.get("challenge").is_some() && body.get("header").is_none());
    if is_challenge {
        let parsed: ChallengeBody = serde_json::from_value(body.clone()).unwrap_or(ChallengeBody {
            challenge: None,
            token: None,
            typ: None,
            encrypt: None,
        });
        if let Some(cfg) = FeishuConfig::from_env() {
            if let Some(expected) = cfg.verification_token.as_deref() {
                if let Some(got) = parsed.token.as_deref() {
                    if got != expected {
                        return (
                            StatusCode::UNAUTHORIZED,
                            Json(json!({ "error": "verification token mismatch" })),
                        )
                            .into_response();
                    }
                }
            }
        }
        let challenge = parsed
            .challenge
            .or_else(|| {
                body.get("challenge")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            })
            .unwrap_or_default();
        info!("feishu url_verification ok");
        return (StatusCode::OK, Json(json!({ "challenge": challenge }))).into_response();
    }

    // Schema 2.0: header.event_type
    let event_type = body
        .pointer("/header/event_type")
        .and_then(|v| v.as_str())
        .or_else(|| body.get("type").and_then(|v| v.as_str()))
        .unwrap_or("");

    if let Some(cfg) = FeishuConfig::from_env() {
        if let Some(expected) = cfg.verification_token.as_deref() {
            let token = body
                .pointer("/header/token")
                .and_then(|v| v.as_str())
                .or_else(|| body.get("token").and_then(|v| v.as_str()));
            if let Some(got) = token {
                if got != expected {
                    return (
                        StatusCode::UNAUTHORIZED,
                        Json(json!({ "error": "verification token mismatch" })),
                    )
                        .into_response();
                }
            }
        }
    }

    if event_type != "im.message.receive_v1" {
        info!(event_type, "feishu event ignored");
        return (StatusCode::OK, Json(json!({ "code": 0 }))).into_response();
    }

    let message = body.pointer("/event/message").cloned().unwrap_or(Value::Null);
    let message_id = message
        .get("message_id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let msg_type = message
        .get("message_type")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if message_id.is_empty() {
        warn!("feishu message missing message_id");
        return (StatusCode::OK, Json(json!({ "code": 0 }))).into_response();
    }
    if msg_type != "text" {
        info!(msg_type, "feishu non-text ignored");
        return (StatusCode::OK, Json(json!({ "code": 0 }))).into_response();
    }

    let text = extract_text(&message).unwrap_or_default();
    let text = text.trim().to_string();
    if text.is_empty() {
        return (StatusCode::OK, Json(json!({ "code": 0 }))).into_response();
    }

    // Ack immediately; Feishu expects a fast 200. Run agent in background.
    let pool = Pool::from_ref(&state);
    tokio::spawn(async move {
        if let Err(e) = process_message(pool, message_id, text).await {
            error!(error = %e, "feishu agent reply failed");
        }
    });

    (StatusCode::OK, Json(json!({ "code": 0 }))).into_response()
}

fn extract_text(message: &Value) -> Option<String> {
    let content = message.get("content")?.as_str()?;
    let v: Value = serde_json::from_str(content).ok()?;
    v.get("text")?.as_str().map(|s| s.to_string())
}

async fn process_message(pool: Pool, message_id: String, text: String) -> Result<(), String> {
    let cfg = FeishuConfig::from_env().ok_or_else(|| {
        "FEISHU_APP_ID / FEISHU_APP_SECRET missing — cannot reply".to_string()
    })?;

    // Optional quick ack so the user sees progress on slow Cursor path.
    let _ = reply_text(&cfg, &message_id, "收到，正在查询…").await;

    let service = AgentService::new(pool);
    let res = service
        .chat(ChatRequest {
            question: text,
            slug: None,
            post_id: None,
            history: Vec::<ChatTurn>::new(),
            provider: None,
            model: None,
        })
        .await
        .map_err(|e| e.to_string())?;

    reply_text(&cfg, &message_id, &res.msg).await?;
    info!(source = %res.source, "feishu replied");
    Ok(())
}

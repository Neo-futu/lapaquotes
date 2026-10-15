//! Meta WhatsApp webhook: GET verify + POST messages → AgentService.

use axum::{
    Json, Router,
    extract::{FromRef, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use cornucopia::deadpool_postgres::Pool;
use serde::Deserialize;
use serde_json::{Value, json};
use tracing::{error, info, warn};

use super::api::{WhatsAppConfig, send_text};
use crate::{
    api::{ChatRequest, ChatTurn},
    service::AgentService,
};

pub fn whatsapp_router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    Pool: FromRef<S>,
{
    Router::new().route(
        "/webhook",
        get(verify_webhook).post(handle_webhook::<S>),
    )
}

#[derive(Debug, Deserialize)]
pub struct VerifyQuery {
    #[serde(rename = "hub.mode")]
    mode: Option<String>,
    #[serde(rename = "hub.verify_token")]
    verify_token: Option<String>,
    #[serde(rename = "hub.challenge")]
    challenge: Option<String>,
}

/// Meta sends GET to verify the callback URL.
async fn verify_webhook(Query(q): Query<VerifyQuery>) -> Response {
    let Some(cfg) = WhatsAppConfig::from_env() else {
        warn!("whatsapp verify: WHATSAPP_* env not configured");
        return (StatusCode::SERVICE_UNAVAILABLE, "whatsapp not configured").into_response();
    };
    if q.mode.as_deref() == Some("subscribe")
        && q.verify_token.as_deref() == Some(cfg.verify_token.as_str())
    {
        let challenge = q.challenge.unwrap_or_default();
        info!("whatsapp webhook verified");
        return (StatusCode::OK, challenge).into_response();
    }
    warn!("whatsapp webhook verify failed");
    (StatusCode::FORBIDDEN, "verification failed").into_response()
}

async fn handle_webhook<S>(
    State(state): State<S>,
    Json(body): Json<Value>,
) -> impl IntoResponse
where
    S: Clone + Send + Sync + 'static,
    Pool: FromRef<S>,
{
    // Always 200 quickly so Meta does not retry aggressively.
    let pool = Pool::from_ref(&state);
    tokio::spawn(async move {
        if let Err(e) = process_payload(pool, body).await {
            error!(error = %e, "whatsapp process failed");
        }
    });
    (StatusCode::OK, Json(json!({ "status": "ok" })))
}

async fn process_payload(pool: Pool, body: Value) -> Result<(), String> {
    let object = body.get("object").and_then(|v| v.as_str()).unwrap_or("");
    if object != "whatsapp_business_account" && object != "page" {
        // Still accept if entry/messages present (some payloads omit object in tests).
        if body.pointer("/entry/0/changes/0/value/messages").is_none() {
            info!(object, "whatsapp ignored non-message object");
            return Ok(());
        }
    }

    let Some(cfg) = WhatsAppConfig::from_env() else {
        return Err("WHATSAPP_ACCESS_TOKEN / PHONE_NUMBER_ID / VERIFY_TOKEN missing".into());
    };

    let entries = body
        .get("entry")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    for entry in entries {
        let changes = entry
            .get("changes")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        for change in changes {
            let value = change.get("value").cloned().unwrap_or(Value::Null);
            // Skip status-only updates (delivered/read).
            let messages = value
                .get("messages")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            for msg in messages {
                handle_one_message(&cfg, pool.clone(), &msg).await?;
            }
        }
    }
    Ok(())
}

async fn handle_one_message(
    cfg: &WhatsAppConfig,
    pool: Pool,
    msg: &Value,
) -> Result<(), String> {
    let from = msg
        .get("from")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    if from.is_empty() {
        return Ok(());
    }
    if !cfg.allows(&from) {
        warn!(%from, "whatsapp sender not in WHATSAPP_ALLOWED_FROM");
        let _ = send_text(cfg, &from, "Sorry, this number is not authorized.").await;
        return Ok(());
    }

    let msg_type = msg.get("type").and_then(|v| v.as_str()).unwrap_or("");
    if msg_type != "text" {
        info!(msg_type, %from, "whatsapp non-text ignored");
        let _ = send_text(
            cfg,
            &from,
            "目前只支持文字消息。Please send text only.",
        )
        .await;
        return Ok(());
    }

    let text = msg
        .pointer("/text/body")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    if text.is_empty() {
        return Ok(());
    }

    let _ = send_text(cfg, &from, "收到，正在查询… / Got it, working…").await;

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

    send_text(cfg, &from, &res.msg).await?;
    info!(%from, source = %res.source, "whatsapp replied");
    Ok(())
}

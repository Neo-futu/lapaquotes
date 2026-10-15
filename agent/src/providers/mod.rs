//! Pluggable LLM backends for the Lapa agent.
//!
//! | Provider | Env | Protocol |
//! |----------|-----|----------|
//! | [`openai`] | `OPENAI_API_KEY` (+ optional `OPENAI_BASE_URL`) | OpenAI-compatible (OpenRouter / OpenAI / … via Rig) |
//! | [`cursor`] | `CURSOR_API_KEY` (`crsr_…`) | Cursor Cloud Agents REST (create agent → poll run) |
//!
//! Selection: per-request `provider`, else `LLM_PROVIDER=openai|cursor|auto`
//! (default `auto`: Cursor if key set, else OpenAI-compatible).

pub mod cursor;
pub mod openai;

use cornucopia::deadpool_postgres::Pool;

use crate::api::{ChatRequest, ChatResponse};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderKind {
    OpenAi,
    Cursor,
}

impl ProviderKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::OpenAi => "openai",
            Self::Cursor => "cursor",
        }
    }

    pub fn parse(s: &str) -> Result<Self, String> {
        match s.trim().to_ascii_lowercase().as_str() {
            "openai" | "rig" | "openrouter" => Ok(Self::OpenAi),
            "cursor" | "cursor-cloud" => Ok(Self::Cursor),
            other => Err(format!(
                "Unknown provider {other:?}. Use openai | openrouter | cursor."
            )),
        }
    }
}

/// Resolve which backend to use from env (and available keys).
pub fn resolve_provider() -> Result<ProviderKind, String> {
    let explicit = std::env::var("LLM_PROVIDER")
        .unwrap_or_else(|_| "auto".into())
        .trim()
        .to_ascii_lowercase();
    match explicit.as_str() {
        "openai" | "rig" | "openrouter" => Ok(ProviderKind::OpenAi),
        "cursor" | "cursor-cloud" => Ok(ProviderKind::Cursor),
        "auto" | "" => {
            // Prefer Cursor when both keys exist (Chat default).
            if cursor::api_key_from_env().is_some() {
                Ok(ProviderKind::Cursor)
            } else if openai::api_key_from_env().is_some() {
                Ok(ProviderKind::OpenAi)
            } else {
                Err(
                    "No LLM configured. Set CURSOR_API_KEY or OPENAI_API_KEY (OpenRouter/OpenAI)."
                        .into(),
                )
            }
        }
        other => Err(format!(
            "Unknown LLM_PROVIDER={other:?}. Use auto | openai | openrouter | cursor."
        )),
    }
}

fn resolve_for_request(req: &ChatRequest) -> Result<ProviderKind, String> {
    if let Some(p) = req.provider.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        return ProviderKind::parse(p);
    }
    resolve_provider()
}

/// Run chat on the active / requested provider.
pub async fn chat(pool: Pool, req: ChatRequest) -> Result<ChatResponse, String> {
    match resolve_for_request(&req)? {
        ProviderKind::Cursor => cursor::chat(pool, req).await,
        ProviderKind::OpenAi => openai::chat(pool, req).await,
    }
}

/// Short UI hint for the active provider (latency / tools).
pub fn provider_ui_hint(kind: ProviderKind) -> &'static str {
    match kind {
        ProviderKind::Cursor => {
            "Cursor Cloud Agents：需创建 agent 并轮询，通常更慢。报价靠服务端 LIVE_DATA 注入（非 Rig tools）。"
        }
        ProviderKind::OpenAi => {
            "OpenRouter / OpenAI 兼容（Rig）：可调用报价与文章 tools，一般更快、按量计费。"
        }
    }
}

/// Label for Chat UI (detect OpenRouter via base URL).
pub fn provider_label(kind: ProviderKind) -> String {
    match kind {
        ProviderKind::Cursor => "cursor".into(),
        ProviderKind::OpenAi => {
            let base = std::env::var("OPENAI_BASE_URL").unwrap_or_default();
            if base.contains("openrouter.ai") {
                "openrouter".into()
            } else {
                "openai".into()
            }
        }
    }
}

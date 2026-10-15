//! Cursor Cloud Agents provider (`crsr_` keys).
//!
//! **Not** OpenAI-compatible: there is no `POST /v1/chat/completions`.
//! Flow: `POST /v1/agents` (no-repo) → poll `GET .../runs/{id}` until FINISHED.
//!
//! Cursor cannot run Rig DynamicTools. We inject a live Quotes DB snapshot into
//! the prompt so answers can cite real institution/price data.

use cornucopia::deadpool_postgres::Pool;
use serde::Deserialize;
use serde_json::json;

use crate::{
    api::{ChatRequest, ChatResponse, ChatTurn},
    context::list_published_posts,
    quote_context::live_quotes_snapshot,
};

const API_ROOT: &str = "https://api.cursor.com";

#[derive(Debug, Deserialize)]
struct CreateResponse {
    agent: AgentRef,
    run: RunRef,
}

#[derive(Debug, Deserialize)]
struct AgentRef {
    id: String,
}

#[derive(Debug, Deserialize)]
struct RunRef {
    id: String,
}

#[derive(Debug, Deserialize)]
struct RunStatus {
    status: String,
    #[serde(default)]
    result: Option<String>,
    #[serde(default)]
    text: Option<String>,
}

pub fn api_key_from_env() -> Option<String> {
    if let Ok(k) = std::env::var("CURSOR_API_KEY") {
        let t = k.trim();
        if !t.is_empty() {
            return Some(t.to_string());
        }
    }
    if let Ok(k) = std::env::var("OPENAI_API_KEY") {
        let t = k.trim();
        if t.starts_with("crsr_") {
            return Some(t.to_string());
        }
    }
    None
}

pub async fn chat(pool: Pool, req: ChatRequest) -> Result<ChatResponse, String> {
    let api_key = api_key_from_env().ok_or_else(|| {
        String::from(
            "CURSOR_API_KEY missing. Create one at cursor.com/dashboard -> API Keys (crsr_...).",
        )
    })?;

    let live_data = load_live_data(&pool).await;
    let msg = complete(&api_key, &req.question, &req.history, &live_data).await?;

    let mut history = req.history.clone();
    history.push(ChatTurn {
        role: "user".into(),
        content: req.question.clone(),
    });
    history.push(ChatTurn {
        role: "assistant".into(),
        content: msg.clone(),
    });

    Ok(ChatResponse {
        msg,
        post_id: None,
        slug: req.slug,
        source: "cursor".into(),
        history,
    })
}

async fn load_live_data(pool: &Pool) -> String {
    let mut parts = Vec::new();
    match live_quotes_snapshot(pool).await {
        Ok(v) => parts.push(format!(
            "QUOTES_SNAPSHOT (same DB as /quotes page):\n{}",
            serde_json::to_string_pretty(&v).unwrap_or_else(|_| v.to_string())
        )),
        Err(e) => parts.push(format!("QUOTES_SNAPSHOT error: {e}")),
    }
    match list_published_posts(pool).await {
        Ok(posts) => {
            let slim: Vec<_> = posts
                .into_iter()
                .take(20)
                .map(|p| json!({ "slug": p.slug, "h1": p.h1 }))
                .collect();
            parts.push(format!(
                "PUBLISHED_POSTS:\n{}",
                serde_json::to_string_pretty(&slim).unwrap_or_default()
            ));
        }
        Err(e) => parts.push(format!("PUBLISHED_POSTS error: {e}")),
    }
    parts.join("\n\n")
}

async fn complete(
    api_key: &str,
    question: &str,
    history: &[ChatTurn],
    live_data: &str,
) -> Result<String, String> {
    let model = std::env::var("CURSOR_MODEL").unwrap_or_else(|_| "auto-smart".into());
    let prompt = build_prompt(question, history, live_data);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(180))
        .build()
        .map_err(|e| format!("http client builder error: {e}"))?;

    let create = client
        .post(format!("{API_ROOT}/v1/agents"))
        .bearer_auth(api_key)
        .json(&json!({
            "prompt": { "text": prompt },
            "model": { "id": model },
            "name": "lapa-chat",
        }))
        .send()
        .await
        .map_err(|e| format!("cursor create agent request failed: {e}"))?;

    let status = create.status();
    let body = create
        .text()
        .await
        .map_err(|e| format!("cursor create agent read body: {e}"))?;
    if !status.is_success() {
        return Err(format!("cursor create agent HTTP {status}: {body}"));
    }

    let created: CreateResponse =
        serde_json::from_str(&body).map_err(|e| format!("cursor create parse: {e}; body={body}"))?;
    let agent_id = created.agent.id;
    let run_id = created.run.id;

    for _ in 0..90 {
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        let resp = client
            .get(format!("{API_ROOT}/v1/agents/{agent_id}/runs/{run_id}"))
            .bearer_auth(api_key)
            .send()
            .await
            .map_err(|e| format!("cursor poll run failed: {e}"))?;
        let st = resp.status();
        let text = resp
            .text()
            .await
            .map_err(|e| format!("cursor poll read body: {e}"))?;
        if !st.is_success() {
            return Err(format!("cursor poll HTTP {st}: {text}"));
        }
        let run: RunStatus =
            serde_json::from_str(&text).map_err(|e| format!("cursor poll parse: {e}; body={text}"))?;
        match run.status.as_str() {
            "FINISHED" => {
                return Ok(run
                    .result
                    .or(run.text)
                    .unwrap_or_else(|| "(empty reply)".into()));
            }
            "FAILED" | "CANCELLED" | "ERROR" => {
                return Err(format!(
                    "cursor run {}: {}",
                    run.status,
                    run.result.or(run.text).unwrap_or_default()
                ));
            }
            _ => continue,
        }
    }
    Err("cursor run timed out waiting for FINISHED".into())
}

fn build_prompt(question: &str, history: &[ChatTurn], live_data: &str) -> String {
    let mut out = String::from(
        "You are the Lapa site assistant for OTC deposit/bond quotes and published posts.\n\
         Answer in the user's language. Be concise and factual.\n\
         IMPORTANT: LIVE_DATA below is real data from the site database (same as the Quotes page).\n\
         When the user asks for latest deposit/bond quotes, pick from LIVE_DATA:\n\
         - use latest_quote_date\n\
         - cite one institution_name and its bid/ask (rates) and term/currency\n\
         Do NOT say you lack Quotes page data if LIVE_DATA contains deposit_quotes_latest.\n\
         If a field is null, say so briefly.\n\n",
    );
    out.push_str("=== LIVE_DATA ===\n");
    out.push_str(live_data);
    out.push_str("\n=== END LIVE_DATA ===\n\n");
    for turn in history {
        out.push_str(&format!("{}: {}\n", turn.role, turn.content));
    }
    out.push_str(&format!("user: {question}\n"));
    out
}

/// Used by `examples/cursor_hello` (no DB snapshot).
pub async fn chat_with_cursor(
    api_key: &str,
    question: &str,
    history: &[ChatTurn],
) -> Result<String, String> {
    complete(api_key, question, history, "(no LIVE_DATA in example)").await
}

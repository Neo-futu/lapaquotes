//! OpenAI-compatible provider via Rig (`sk-…` / OpenRouter + optional base URL).
//!
//! Official Rig pattern (`agent_with_tools` + multi-turn):
//! ```text
//! AgentBuilder → model.completion(id)
//!             → .dynamic_tools([...])   // closures query Postgres
//!             → .add_hook(ForceToolOnFirstTurn)
//!             → agent.prompt(q).max_turns(N)
//! ```
//! Each agent *turn* is one LLM HTTP call; tools run locally between turns.

use cornucopia::deadpool_postgres::Pool;
use rig::message::Message;
use tracing::info;

use crate::{
    api::{ChatRequest, ChatResponse, ChatTurn},
    context::{ContextError, load_post_grounding},
    quote_context::recent_quote_dates,
    rig_agent::{AgentStaticContext, build_post_agent},
};

pub fn api_key_from_env() -> Option<String> {
    std::env::var("OPENAI_API_KEY")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && !s.starts_with("crsr_"))
}

pub async fn chat(pool: Pool, req: ChatRequest) -> Result<ChatResponse, String> {
    if api_key_from_env().is_none() {
        return Err(
            "OPENAI_API_KEY missing or invalid. Use an OpenAI-compatible sk-… key \
             (not a Cursor crsr_ key)."
                .into(),
        );
    }

    let mut focus_missing_slug: Option<String> = None;
    let grounding = if req.post_id.is_some() || req.slug.is_some() {
        match load_post_grounding(&pool, req.post_id.as_deref(), req.slug.as_deref()).await {
            Ok(g) => g,
            Err(ContextError::NotFound) => {
                focus_missing_slug = req.slug.clone().or(req.post_id.clone());
                None
            }
            Err(e) => return Err(e.to_string()),
        }
    } else {
        None
    };

    // Prefetch dates into agent context (agent_with_context) so the model
    // often needs only get_deposit_quotes → answer (2 turns, not 3).
    let static_ctx = match recent_quote_dates(&pool).await {
        Ok(dates) => AgentStaticContext {
            recent_quote_dates: dates
                .into_iter()
                .take(8)
                .map(|d| d.to_string())
                .collect(),
        },
        Err(e) => {
            info!(error = %e, "recent_quote_dates prefetch failed; tools still available");
            AgentStaticContext::default()
        }
    };

    let built = build_post_agent(
        pool,
        grounding.as_ref(),
        focus_missing_slug.as_deref(),
        req.model.as_deref(),
        Some(&static_ctx),
    )?;

    info!(
        max_turns = built.max_turns,
        dates = static_ctx.recent_quote_dates.len(),
        "rig agent ready (official multi-turn tool loop)"
    );

    let (msg, history_out) = if req.history.is_empty() {
        let response = built
            .agent
            .prompt(&req.question)
            .max_turns(built.max_turns)
            .await
            .map_err(|e| e.to_string())?;
        let msg = response.output().to_string();
        let history = vec![
            ChatTurn {
                role: "user".into(),
                content: req.question.clone(),
            },
            ChatTurn {
                role: "assistant".into(),
                content: msg.clone(),
            },
        ];
        (msg, history)
    } else {
        let mut history_msgs: Vec<Message> = req
            .history
            .iter()
            .filter_map(|t| match t.role.as_str() {
                "user" => Some(Message::user(t.content.clone())),
                "assistant" => Some(Message::assistant(t.content.clone())),
                _ => None,
            })
            .collect();

        let response = built
            .agent
            .chat(&req.question, &mut history_msgs)
            .await
            .map_err(|e| e.to_string())?;
        let msg = response.output().to_string();

        let mut history = req.history.clone();
        history.push(ChatTurn {
            role: "user".into(),
            content: req.question.clone(),
        });
        history.push(ChatTurn {
            role: "assistant".into(),
            content: msg.clone(),
        });
        (msg, history)
    };

    let source = {
        let base = std::env::var("OPENAI_BASE_URL").unwrap_or_default();
        if base.contains("openrouter.ai") {
            "openrouter".into()
        } else {
            "openai".into()
        }
    };

    Ok(ChatResponse {
        msg,
        post_id: grounding.as_ref().map(|g| g.id.clone()),
        slug: grounding
            .as_ref()
            .map(|g| g.slug.clone())
            .or(req.slug.clone()),
        source,
        history: history_out,
    })
}

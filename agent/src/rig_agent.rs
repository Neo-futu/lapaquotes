//! Rig agent construction — mirrors official examples:
//! - `agent_with_tools` — `DynamicTool` + `default_max_turns`
//! - `agent_with_context` — static context documents (recent quote dates)
//! - `force_tool_first_turn` — `AgentHook` forces tool on turn 1 only
//! - `multi_turn_agent` — multi-turn tool loop via `max_turns`
//!
//! Flow (OpenRouter / OpenAI-compatible):
//! ```text
//! User prompt
//!   → AgentBuilder (model + preamble + dynamic_tools + context)
//!   → turn 1: model must tool_call (hook) → Rust closure → Postgres
//!   → turn 2+: model may answer with tool results
//! ```

use cornucopia::deadpool_postgres::Pool;
use rig::prelude::*;
use rig::providers::openai;

use crate::context::PostGrounding;
use crate::hooks::ForceToolOnFirstTurn;
use crate::quote_tools::quote_tools;
use crate::tools::post_tools;

const PREAMBLE: &str = "\
You are the Lapa site assistant. Answer using tools for facts — never invent quotes or posts.

Tools:
- Posts: get_post_by_slug, list_published_posts, search_published_posts
- Quotes: list_recent_quote_dates, get_deposit_quotes, get_bond_quotes, \
  search_bond_quotes, list_quote_institutions

Rules:
1. Call tools before answering factual questions. Do not invent rates or articles.
2. For latest deposit/bond quotes: call get_deposit_quotes or get_bond_quotes \
   WITHOUT a date (the tool already returns the latest date). Prefer ONE tool \
   call, then answer. Do not call list_recent_quote_dates unless the user asks \
   for dates or the tool fails.
3. If STATIC CONTEXT already lists recent_quote_dates, use that date when needed.
4. Answer in the user's language. Be concise. Cite institution, date, term, bid/ask.
5. Never invent unpublished posts or quotes not returned by a tool.";

pub struct BuiltAgent {
    pub agent: Agent,
    pub max_turns: usize,
}

/// Extra static context (official `agent_with_context` pattern).
#[derive(Debug, Default)]
pub struct AgentStaticContext {
    /// Newest quote dates from Postgres (ISO strings).
    pub recent_quote_dates: Vec<String>,
}

/// Build the Lapa agent: OpenAI-compatible model + Rig DynamicTools + optional focus.
pub fn build_post_agent(
    pool: Pool,
    focus: Option<&PostGrounding>,
    focus_missing_slug: Option<&str>,
    model_override: Option<&str>,
    static_ctx: Option<&AgentStaticContext>,
) -> Result<BuiltAgent, String> {
    let model_id = model_override
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .or_else(|| {
            std::env::var("OPENAI_MODEL")
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
        })
        .unwrap_or_else(|| "deepseek/deepseek-chat".into());
    let max_turns = std::env::var("AGENT_MAX_TURNS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(4usize);
    let max_tokens = std::env::var("AGENT_MAX_TOKENS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(4096u64);

    let api_key = std::env::var("OPENAI_API_KEY")
        .map(|s| s.trim().to_string())
        .unwrap_or_default();
    if api_key.is_empty() || api_key.starts_with("crsr_") {
        return Err(
            "OPENAI_API_KEY missing or is a Cursor crsr_ key. \
             For Cursor keys set CURSOR_API_KEY (Cloud Agents). \
             For OpenAI-compatible providers set OPENAI_API_KEY=sk-..."
                .into(),
        );
    }
    let mut cfg = openai::OpenAIConfig::new(api_key);
    if let Ok(base) = std::env::var("OPENAI_BASE_URL") {
        let base = base.trim();
        if !base.is_empty() {
            cfg = cfg.with_base_url(base);
        }
    }
    let openai = cfg.client();
    let model = openai.completion(model_id);

    let tools = {
        let mut post = post_tools(pool.clone()).map_err(|e| e.to_string())?;
        post.extend(quote_tools(pool).map_err(|e| e.to_string())?);
        post
    };

    // Official: AgentBuilder + preamble + dynamic_tools + default_max_turns
    // + force_tool_first_turn hook (Required only on turn 1).
    let mut builder = AgentBuilder::new(model)
        .name("lapa-post-agent")
        .description("Lapa quotes/posts assistant using Postgres tools via Rig.")
        .preamble(PREAMBLE)
        .dynamic_tools(tools)
        .max_tokens(max_tokens)
        .default_max_turns(max_turns)
        .add_hook(ForceToolOnFirstTurn);

    // Official agent_with_context: small static docs reduce tool round-trips.
    if let Some(ctx) = static_ctx {
        if !ctx.recent_quote_dates.is_empty() {
            builder = builder.context(format!(
                "STATIC CONTEXT — recent_quote_dates (newest first, from Postgres):\n{}",
                ctx.recent_quote_dates.join("\n")
            ));
        }
    }

    if let Some(g) = focus {
        builder = builder
            .context(format!(
                "Focus post (provided by API caller):\n{}",
                serde_json::to_string_pretty(g).unwrap_or_else(|_| g.slug.clone())
            ))
            .context(format!(
                "Focus post body text:\n{}",
                truncate(&g.content_text, 6000)
            ));
    } else if let Some(slug) = focus_missing_slug {
        builder = builder.context(format!(
            "The API caller asked about slug `{slug}`, but it was not found or not published. \
             Use search_published_posts / list_published_posts to help the user."
        ));
    }

    Ok(BuiltAgent {
        agent: builder.build(),
        max_turns,
    })
}

fn truncate(s: &str, max: usize) -> String {
    let mut it = s.chars();
    let out: String = it.by_ref().take(max).collect();
    if it.next().is_some() {
        format!("{out}…")
    } else {
        out
    }
}

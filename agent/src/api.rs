use serde::{Deserialize, Serialize};

/// One prior turn for multi-turn chat (see Rig `agent.chat` / multi_turn examples).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChatTurn {
    /// `user` or `assistant`
    pub role: String,
    pub content: String,
}

/// Incoming chat / agent question.
#[derive(Debug, Clone, Deserialize)]
pub struct ChatRequest {
    /// User question or instruction.
    pub question: String,
    /// Optional published post slug to ground / focus the answer.
    #[serde(default)]
    pub slug: Option<String>,
    /// Optional post id (takes precedence over slug when both set).
    #[serde(default)]
    pub post_id: Option<String>,
    /// Optional prior turns for multi-turn tool loops.
    #[serde(default)]
    pub history: Vec<ChatTurn>,
    /// Optional backend override: `openai` (OpenRouter/Rig) | `cursor`. Empty = env `LLM_PROVIDER`.
    #[serde(default)]
    pub provider: Option<String>,
    /// Optional OpenRouter/OpenAI model id (e.g. `deepseek/deepseek-chat`). Ignored for Cursor.
    #[serde(default)]
    pub model: Option<String>,
}

/// Standard agent reply envelope.
#[derive(Debug, Clone, Serialize)]
pub struct ChatResponse {
    pub msg: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    pub source: String,
    /// Echo of history after the run (includes this turn) for clients that persist chat.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub history: Vec<ChatTurn>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub service: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct ErrorBody {
    pub error: String,
}

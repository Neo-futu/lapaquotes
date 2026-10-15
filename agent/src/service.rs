use cornucopia::deadpool_postgres::Pool;

use crate::{
    api::{ChatRequest, ChatResponse},
    context::ContextError,
    providers,
};

#[derive(Clone)]
pub struct AgentService {
    pool: Pool,
}

impl AgentService {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    pub async fn chat(&self, req: ChatRequest) -> Result<ChatResponse, AgentError> {
        providers::chat(self.pool.clone(), req)
            .await
            .map_err(AgentError::Provider)
    }

    /// Which backend `auto` would pick right now (for UI / health).
    pub fn active_provider(&self) -> Result<&'static str, AgentError> {
        providers::resolve_provider()
            .map(|k| k.as_str())
            .map_err(AgentError::Provider)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    #[error(transparent)]
    Context(#[from] ContextError),
    #[error("llm provider error: {0}")]
    Provider(String),
}

use axum::{
    Json, Router,
    extract::{FromRef, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use cornucopia::deadpool_postgres::Pool;

use crate::{
    api::{ChatRequest, ErrorBody, HealthResponse},
    bi, channels, mcp,
    service::AgentService,
};

/// Standalone process state.
#[derive(Clone)]
pub struct AgentState {
    pub pool: Pool,
}

impl AgentState {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

impl FromRef<AgentState> for Pool {
    fn from_ref(state: &AgentState) -> Self {
        state.pool.clone()
    }
}

/// Nest at `/agent` on site, or mount at `/` with [`AgentState`] standalone.
///
/// Routes:
/// - `GET  /health`
/// - `POST /chat`
/// - `GET  /mcp/status`
/// - `POST /channels/feishu/events`
/// - `GET|POST /channels/whatsapp/webhook`
/// - `/bi/*` — BI catalog / migrate / sync / verify / facts / jobs
pub fn router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    Pool: FromRef<S>,
{
    Router::new()
        .route("/health", get(health))
        .route("/chat", post(chat::<S>))
        .route("/mcp/status", get(mcp_status))
        .nest("/bi", bi::http::bi_router::<S>())
        .nest("/channels/feishu", channels::feishu::feishu_router::<S>())
        .nest(
            "/channels/whatsapp",
            channels::whatsapp::whatsapp_router::<S>(),
        )
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "agent",
    })
}

async fn mcp_status() -> impl IntoResponse {
    Json(mcp::mcp_status())
}

async fn chat<S>(State(state): State<S>, Json(req): Json<ChatRequest>) -> impl IntoResponse
where
    S: Send + Sync,
    Pool: FromRef<S>,
{
    let pool = Pool::from_ref(&state);
    let service = AgentService::new(pool);
    match service.chat(req).await {
        Ok(res) => (StatusCode::OK, Json(res)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorBody {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

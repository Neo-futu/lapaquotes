//! AI-native agent for Lapa.
//!
//! # Layout
//!
//! ```text
//! agent/src/
//!   api.rs              HTTP contracts (ChatRequest / Response)
//!   providers/          openai (Rig + tools) | cursor (Cloud Agents)
//!   tools.rs            Rig DynamicTool — posts
//!   quote_tools.rs      Rig DynamicTool — OTC quotes
//!   rig_agent.rs        AgentBuilder (official Rig patterns)
//!   hooks.rs            ForceToolOnFirstTurn
//!   mcp/                Rig MCP client surface (optional remote tools)
//!   channels/           External messengers → AgentService
//!     feishu/           飞书机器人事件 Webhook
//!     whatsapp/         WhatsApp Cloud API Webhook
//!   bi/                 BI 层：平台报价 → BI 库表 → 飞书多维表格
//!     store/            BiQuoteFact / BiBitable* / BiSyncJob
//!     bitable/          Feishu 适配器（发现表/视图 + 推送）
//!     service.rs        BiService 编排
//!     http.rs           /bi/*
//!   service.rs          AgentService::chat
//!   router.rs           /health /chat /mcp /channels /bi
//! ```
//!
//! # Request flow
//!
//! ```text
//! Web /chat  ──┐
//! POST /chat ──┼→ AgentService → providers::{openai|cursor}
//! 飞书 / WA ───┘
//!
//! Platform DepositQuote/BondQuote
//!        → bi::extract → BiQuoteFact
//!        → bi::bitable → Feishu Base（运行时发现 TABLE_ID）
//! ```

pub mod api;
pub mod bi;
pub mod channels;
pub mod context;
pub mod hooks;
pub mod mcp;
pub mod providers;
pub mod quote_context;
pub mod quote_tools;
pub mod rig_agent;
pub mod router;
pub mod service;
pub mod tools;

pub use api::{ChatRequest, ChatResponse, ChatTurn, HealthResponse};
pub use providers::{ProviderKind, provider_label, provider_ui_hint, resolve_provider};
pub use router::{AgentState, router};
pub use service::AgentService;

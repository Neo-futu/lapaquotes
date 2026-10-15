//! Outbound chat **channels** — adapters that turn external messengers into
//! `AgentService::chat` calls.
//!
//! ```text
//! Feishu / WhatsApp user
//!   → POST /agent/channels/{feishu|whatsapp}/…
//!   → AgentService::chat  (same as web /chat)
//!   → platform reply API
//! ```
//!
//! These are **not** MCP. MCP lives in `crate::mcp`.

pub mod feishu;
pub mod whatsapp;

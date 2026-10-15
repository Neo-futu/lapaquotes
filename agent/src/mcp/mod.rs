//! Rig **MCP** surface for Lapa.
//!
//! # Does Rig support MCP?
//!
//! Yes. Official Rig exposes MCP via the `rmcp` feature (`rig-rmcp` crate):
//! remote MCP servers become portable tools that the Agent can call —
//! same multi-turn loop as our local `DynamicTool`s.
//!
//! Enable in `agent/Cargo.toml`:
//! ```toml
//! rig = { version = "0.43", features = ["rmcp"] }
//! ```
//! and this crate's feature `mcp`.
//!
//! # MCP vs 飞书机器人
//!
//! | | MCP | 飞书 |
//! |--|-----|------|
//! | 协议 | Model Context Protocol（给 LLM 挂工具） | 开放平台事件 Webhook |
//! | 用途 | Agent ←→ 外部工具服务 | 用户在飞书里和机器人聊天 |
//! | 本仓库 | `agent::mcp` | `agent::channels::feishu` |
//!
//! 飞书**不**讲 MCP。飞书机器人应走 `channels::feishu` → `AgentService::chat`
//!（与 `/chat`、`POST /agent/chat` 同一套问答）。

pub mod client;
pub mod config;

pub use client::{McpStatus, describe_attach_plan, status as mcp_status};
pub use config::McpConfig;

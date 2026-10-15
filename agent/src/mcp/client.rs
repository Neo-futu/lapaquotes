//! Optional MCP **client**: attach remote MCP server tools to the Rig agent.
//!
//! Full `rig::tool::rmcp` wiring is compiled only with `--features mcp`
//! (requires `rig` feature `rmcp`). Until a server command is configured,
//! [`status`] reports readiness and local DynamicTools still power chat.

use super::config::McpConfig;

#[derive(Debug, Clone, serde::Serialize)]
pub struct McpStatus {
    pub feature_enabled: bool,
    pub config_ready: bool,
    pub transport: String,
    pub server_cmd: Option<String>,
    pub note: String,
}

pub fn status() -> McpStatus {
    let cfg = McpConfig::from_env();
    #[cfg(feature = "mcp")]
    let feature_enabled = true;
    #[cfg(not(feature = "mcp"))]
    let feature_enabled = false;

    let note = if !feature_enabled {
        "Compile with agent feature `mcp` (rig/rmcp) to attach remote MCP tools.".into()
    } else if !cfg.is_ready() {
        "Set MCP_ENABLED=1 and MCP_SERVER_CMD (+ MCP_SERVER_ARGS) to connect.".into()
    } else {
        "MCP client configured. Tools attach on the OpenRouter/Rig agent path when wired.".into()
    };

    McpStatus {
        feature_enabled,
        config_ready: cfg.is_ready(),
        transport: cfg.transport,
        server_cmd: cfg.server_cmd,
        note,
    }
}

/// Placeholder for attaching MCP tools to an agent builder.
/// Local Postgres tools in `crate::tools` / `crate::quote_tools` always remain available.
pub fn describe_attach_plan() -> String {
    let s = status();
    format!(
        "mcp feature={} ready={} transport={} cmd={:?} — {}",
        s.feature_enabled, s.config_ready, s.transport, s.server_cmd, s.note
    )
}

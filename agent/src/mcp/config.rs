//! MCP client configuration from environment.
//!
//! ```text
//! MCP_ENABLED=1
//! MCP_TRANSPORT=stdio          # stdio | sse (sse reserved)
//! MCP_SERVER_CMD=npx
//! MCP_SERVER_ARGS=-y,@modelcontextprotocol/server-filesystem,/data
//! ```

#[derive(Debug, Clone, Default)]
pub struct McpConfig {
    pub enabled: bool,
    /// `stdio` today; `sse` reserved for remote MCP HTTP/SSE servers.
    pub transport: String,
    pub server_cmd: Option<String>,
    pub server_args: Vec<String>,
}

impl McpConfig {
    pub fn from_env() -> Self {
        let enabled = matches!(
            std::env::var("MCP_ENABLED")
                .unwrap_or_default()
                .trim()
                .to_ascii_lowercase()
                .as_str(),
            "1" | "true" | "yes" | "on"
        );
        let transport = std::env::var("MCP_TRANSPORT")
            .unwrap_or_else(|_| "stdio".into())
            .trim()
            .to_ascii_lowercase();
        let server_cmd = std::env::var("MCP_SERVER_CMD")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        let server_args = std::env::var("MCP_SERVER_ARGS")
            .ok()
            .map(|s| {
                s.split(',')
                    .map(|p| p.trim().to_string())
                    .filter(|p| !p.is_empty())
                    .collect()
            })
            .unwrap_or_default();
        Self {
            enabled,
            transport,
            server_cmd,
            server_args,
        }
    }

    pub fn is_ready(&self) -> bool {
        self.enabled && self.server_cmd.is_some()
    }
}

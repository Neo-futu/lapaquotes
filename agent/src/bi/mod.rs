//! BI 层：平台报价 → BI 库表 → 飞书多维表格。
//!
//! # 分层
//!
//! ```text
//! Platform DB (DepositQuote / BondQuote)
//!        │ extract
//!        ▼
//! BI Store   BiQuoteFact / BiBitable* / BiSyncJob
//!        │
//!        ▼
//! BiService  编排：catalog / migrate / sync_day / verify
//!        │
//!        ▼
//! bitable/   Feishu 适配器（仅需 ACCESS_TOKEN + APP_TOKEN + API_BASE）
//!            运行时发现 TABLE_ID / VIEW_ID / TABLE_NAME
//! ```
//!
//! HTTP：`/agent/bi/*`（见 [`http`]）。
//! 日更脚本：`scripts/bi/daily_sync.py`。

pub mod bitable;
pub mod error;
pub mod extract;
pub mod http;
pub mod models;
pub mod service;
pub mod store;

pub use error::BiError;
pub use models::{BitableRole, QuoteFact, SyncJob, TableCatalog};
pub use service::BiService;

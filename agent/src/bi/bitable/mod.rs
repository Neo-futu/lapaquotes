//! Feishu Bitable adapter.
//!
//! Env（仅此三项）：
//! - `FEISHU_BITABLE_ACCESS_TOKEN` — `pt-…` personalBaseToken
//! - `FEISHU_BITABLE_APP_TOKEN` — Base app_token
//! - `FEISHU_BITABLE_API_BASE` — 可选；`pt-` 默认 `https://base-api.feishu.cn`
//!
//! TABLE_ID / VIEW_ID / TABLE_NAME 由 [`discovery`] 拉取后写入 BI catalog。

mod client;
mod config;
mod dashboard;
mod discovery;
mod migrate;
mod push;

pub use client::{BitableClient, RemoteRecord};
pub use config::BitableConfig;
pub use dashboard::{
    ensure_ledger_chart_views, ensure_ledger_chart_views_for_date, ensure_otc_dashboard,
    list_dashboards, DashboardEnsureReport, DashboardInfo, LedgerViewsReport,
    DEFAULT_OTC_DASHBOARD_NAME, DEFAULT_SOURCE_DASHBOARD_ID,
};
pub use discovery::{assign_roles, discover_base};
pub use migrate::{ensure_otc_fields, migrate_all_tables, MigrateReport};
pub use push::{push_digest, push_ledger, LedgerPushReport};

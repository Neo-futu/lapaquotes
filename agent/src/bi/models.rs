//! BI domain models (independent of Feishu wire types).

use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BitableRole {
    /// 明细报价台账（逐条存款/债券）
    QuoteLedger,
    /// 推送摘要表（按机构/日汇总文案）
    QuoteDigest,
    /// 每日最高存款报价（与站点 Best Deposit Quote 同口径）
    DailyBest,
    Unknown,
}

impl BitableRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::QuoteLedger => "quote_ledger",
            Self::QuoteDigest => "quote_digest",
            Self::DailyBest => "daily_best",
            Self::Unknown => "unknown",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s {
            "quote_ledger" => Self::QuoteLedger,
            "quote_digest" => Self::QuoteDigest,
            "daily_best" => Self::DailyBest,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableCatalog {
    pub table_id: String,
    pub name: String,
    pub role: BitableRole,
    pub default_view_id: Option<String>,
    pub revision: Option<i32>,
    pub views: Vec<ViewCatalog>,
    pub fields: Vec<FieldCatalog>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewCatalog {
    pub view_id: String,
    pub name: String,
    pub view_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldCatalog {
    pub field_id: String,
    pub name: String,
    pub field_type: i32,
    pub ui_type: Option<String>,
    pub is_primary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteFact {
    pub id: String,
    pub sync_key: String,
    pub quote_date: NaiveDate,
    /// Next HK working day after `quote_date` (skip Sat/Sun + `Holiday` region=HK).
    pub next_workday: Option<NaiveDate>,
    /// `deposit` | `bond`
    pub kind: String,
    pub institution_name: String,
    pub institution_cn_name: Option<String>,
    pub term: Option<String>,
    /// Numeric tenor order (days-ish); used for Feishu Number field「期限排序」.
    pub term_sort: Option<i32>,
    pub currency: Option<String>,
    pub bond_code: Option<String>,
    pub bond_name: Option<String>,
    pub bid: Option<f64>,
    pub ask: Option<f64>,
    pub note: Option<String>,
    pub bitable_table_id: Option<String>,
    pub bitable_record_id: Option<String>,
    pub pushed_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncJob {
    pub id: String,
    pub job_type: String,
    pub quote_date: Option<NaiveDate>,
    pub status: String,
    pub created_n: i32,
    pub updated_n: i32,
    pub deleted_n: i32,
    pub facts_n: i32,
    pub error: Option<String>,
    pub detail: Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct SyncDayReport {
    pub job_id: String,
    pub date: String,
    pub facts: usize,
    pub created: usize,
    pub updated: usize,
    pub deleted: usize,
    pub digest_updated: usize,
    pub best_updated: usize,
    pub ledger_table_id: String,
    pub digest_table_id: Option<String>,
    pub best_table_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct VerifyReport {
    pub ok: bool,
    pub catalog_tables: usize,
    pub sync: SyncDayReport,
    pub read_back: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardCatalog {
    pub block_id: String,
    pub name: String,
    pub role: String,
    pub url: String,
    pub source_block_id: Option<String>,
}

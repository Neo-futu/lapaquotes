//! Quote reads for the Rig agent — mirrors `context.rs` post helpers but for
//! OTC deposit / bond quotes. Reuses [`ContextError`].

use cornucopia::{deadpool_postgres::Pool, queries};
use chrono::NaiveDate;
use serde::Serialize;
use std::convert::TryInto;

use crate::context::ContextError;

/// Compact summary of a deposit quote row.
#[derive(Debug, Clone, Serialize)]
pub struct DepositQuoteSummary {
    pub quote_date: NaiveDate,
    pub institution_name: String,
    pub institution_cn_name: Option<String>,
    pub term: String,
    pub term_sort: i32,
    pub currency: String,
    pub bid: Option<f64>,
    pub ask: Option<f64>,
    pub note: Option<String>,
}

/// Compact summary of a bond quote row.
#[derive(Debug, Clone, Serialize)]
pub struct BondQuoteSummary {
    pub quote_date: NaiveDate,
    pub institution_name: String,
    pub institution_cn_name: Option<String>,
    pub bond_code: String,
    pub bond_name: Option<String>,
    pub currency: String,
    pub maturity_date: Option<NaiveDate>,
    pub coupon: Option<f64>,
    pub bid_yield: Option<f64>,
    pub ask_yield: Option<f64>,
    pub bid_price: Option<f64>,
    pub ask_price: Option<f64>,
    pub direction: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct InstitutionSummary {
    pub name: String,
    pub short_name: Option<String>,
    pub email: Option<String>,
    pub is_active: bool,
}

fn dec_to_f64(d: Option<rust_decimal::Decimal>) -> Option<f64> {
    d.and_then(|d| d.try_into().ok())
}

pub async fn recent_quote_dates(pool: &Pool) -> Result<Vec<NaiveDate>, ContextError> {
    let client = pool
        .get()
        .await
        .map_err(|e| ContextError::Pool(e.to_string()))?;
    let rows = queries::agent_quote::recent_quote_dates()
        .bind(&client)
        .all()
        .await?;
    Ok(rows)
}

pub async fn deposit_by_date(
    pool: &Pool,
    quote_date: Option<NaiveDate>,
    institution_name: Option<String>,
) -> Result<Vec<DepositQuoteSummary>, ContextError> {
    let client = pool
        .get()
        .await
        .map_err(|e| ContextError::Pool(e.to_string()))?;
    let rows = queries::agent_quote::deposit_by_date()
        .bind(&client, &quote_date, &institution_name)
        .all()
        .await?;
    Ok(rows
        .into_iter()
        .map(|r| DepositQuoteSummary {
            quote_date: r.quote_date,
            institution_name: r.institution_name,
            institution_cn_name: r.institution_cn_name,
            term: r.term,
            term_sort: r.term_sort,
            currency: r.currency,
            bid: dec_to_f64(r.bid_rate),
            ask: dec_to_f64(r.ask_rate),
            note: r.note,
        })
        .collect())
}

pub async fn bond_by_date(
    pool: &Pool,
    quote_date: Option<NaiveDate>,
    institution_name: Option<String>,
) -> Result<Vec<BondQuoteSummary>, ContextError> {
    let client = pool
        .get()
        .await
        .map_err(|e| ContextError::Pool(e.to_string()))?;
    let rows = queries::agent_quote::bond_by_date()
        .bind(&client, &quote_date, &institution_name)
        .all()
        .await?;
    Ok(rows
        .into_iter()
        .map(|r| BondQuoteSummary {
            quote_date: r.quote_date,
            institution_name: r.institution_name,
            institution_cn_name: r.institution_cn_name,
            bond_code: r.bond_code,
            bond_name: r.bond_name,
            currency: r.currency,
            maturity_date: r.maturity_date,
            coupon: dec_to_f64(r.coupon_rate),
            bid_yield: dec_to_f64(r.bid_yield),
            ask_yield: dec_to_f64(r.ask_yield),
            bid_price: dec_to_f64(r.bid_price),
            ask_price: dec_to_f64(r.ask_price),
            direction: r.direction,
        })
        .collect())
}

pub async fn search_bonds(
    pool: &Pool,
    query: &str,
) -> Result<Vec<BondQuoteSummary>, ContextError> {
    let q = query.trim();
    let query_opt = if q.is_empty() {
        None
    } else {
        Some(q.to_string())
    };
    let client = pool
        .get()
        .await
        .map_err(|e| ContextError::Pool(e.to_string()))?;
    let rows = queries::agent_quote::search_bonds()
        .bind(&client, &query_opt)
        .all()
        .await?;
    Ok(rows
        .into_iter()
        .map(|r| BondQuoteSummary {
            quote_date: r.quote_date,
            institution_name: r.institution_name,
            institution_cn_name: None,
            bond_code: r.bond_code,
            bond_name: r.bond_name,
            currency: r.currency,
            maturity_date: r.maturity_date,
            coupon: dec_to_f64(r.coupon_rate),
            bid_yield: dec_to_f64(r.bid_yield),
            ask_yield: dec_to_f64(r.ask_yield),
            bid_price: dec_to_f64(r.bid_price),
            ask_price: dec_to_f64(r.ask_price),
            direction: r.direction,
        })
        .collect())
}

pub async fn institutions(pool: &Pool) -> Result<Vec<InstitutionSummary>, ContextError> {
    let client = pool
        .get()
        .await
        .map_err(|e| ContextError::Pool(e.to_string()))?;
    let rows = queries::agent_quote::institutions()
        .bind(&client)
        .all()
        .await?;
    Ok(rows
        .into_iter()
        .map(|r| InstitutionSummary {
            name: r.name,
            short_name: r.short_name,
            email: r.email,
            is_active: r.is_active,
        })
        .collect())
}

/// Compact DB snapshot for providers that cannot call Rig tools (e.g. Cursor).
/// Includes recent dates + latest deposit/bond rows (capped).
pub async fn live_quotes_snapshot(pool: &Pool) -> Result<serde_json::Value, ContextError> {
    let dates = recent_quote_dates(pool).await?;
    let latest = dates.first().copied();
    let deposits = if latest.is_some() {
        let mut rows = deposit_by_date(pool, latest, None).await?;
        rows.truncate(48);
        rows
    } else {
        Vec::new()
    };
    let bonds = if latest.is_some() {
        let mut rows = bond_by_date(pool, latest, None).await?;
        rows.truncate(24);
        rows
    } else {
        Vec::new()
    };
    Ok(serde_json::json!({
        "recent_quote_dates": dates.iter().take(12).map(|d| d.to_string()).collect::<Vec<_>>(),
        "latest_quote_date": latest.map(|d| d.to_string()),
        "deposit_quotes_latest": deposits,
        "bond_quotes_latest": bonds,
        "note": "Data is from the same Postgres tables as the Quotes page. bid/ask for deposits are rates.",
    }))
}

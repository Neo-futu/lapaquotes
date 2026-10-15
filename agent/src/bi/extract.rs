//! Extract platform OTC quotes → BI [`QuoteFact`] rows.

use chrono::NaiveDate;
use cornucopia::deadpool_postgres::Pool;

use super::error::BiError;
use super::models::QuoteFact;
use super::store::new_id;
use crate::quote_context::{bond_by_date, deposit_by_date};

pub fn deposit_sync_key(d: NaiveDate, inst: &str, term: &str, ccy: &str) -> String {
    format!("D|{d}|{inst}|{term}|{ccy}")
}

pub fn bond_sync_key(d: NaiveDate, inst: &str, code: &str, ccy: &str) -> String {
    format!("B|{d}|{inst}|{code}|{ccy}")
}

pub async fn extract_day(pool: &Pool, date: NaiveDate) -> Result<Vec<QuoteFact>, BiError> {
    let deposits = deposit_by_date(pool, Some(date), None)
        .await
        .map_err(|e| BiError::Msg(e.to_string()))?;
    let bonds = bond_by_date(pool, Some(date), None)
        .await
        .map_err(|e| BiError::Msg(e.to_string()))?;

    let mut facts = Vec::with_capacity(deposits.len() + bonds.len());
    for d in deposits {
        facts.push(QuoteFact {
            id: new_id(),
            sync_key: deposit_sync_key(d.quote_date, &d.institution_name, &d.term, &d.currency),
            quote_date: d.quote_date,
            kind: "deposit".into(),
            institution_name: d.institution_name,
            term: Some(d.term),
            currency: Some(d.currency),
            bond_code: None,
            bond_name: None,
            bid: d.bid,
            ask: d.ask,
            note: d.note,
            bitable_table_id: None,
            bitable_record_id: None,
            pushed_at: None,
        });
    }
    for b in bonds {
        let bid = b.bid_yield.or(b.bid_price);
        let ask = b.ask_yield.or(b.ask_price);
        let mut note_parts = Vec::new();
        if let Some(d) = &b.direction {
            note_parts.push(format!("方向:{d}"));
        }
        if let Some(c) = b.coupon {
            note_parts.push(format!("票息:{c}"));
        }
        if let Some(md) = b.maturity_date {
            note_parts.push(format!("到期:{md}"));
        }
        facts.push(QuoteFact {
            id: new_id(),
            sync_key: bond_sync_key(b.quote_date, &b.institution_name, &b.bond_code, &b.currency),
            quote_date: b.quote_date,
            kind: "bond".into(),
            institution_name: b.institution_name,
            term: None,
            currency: Some(b.currency),
            bond_code: Some(b.bond_code),
            bond_name: b.bond_name,
            bid,
            ask,
            note: if note_parts.is_empty() {
                None
            } else {
                Some(note_parts.join(" · "))
            },
            bitable_table_id: None,
            bitable_record_id: None,
            pushed_at: None,
        });
    }
    Ok(facts)
}

pub async fn resolve_quote_date(
    pool: &Pool,
    date: Option<NaiveDate>,
) -> Result<NaiveDate, BiError> {
    if let Some(d) = date {
        return Ok(d);
    }
    let dates = crate::quote_context::recent_quote_dates(pool)
        .await
        .map_err(|e| BiError::Msg(e.to_string()))?;
    dates
        .into_iter()
        .next()
        .ok_or_else(|| BiError::Msg("no quote dates in platform DB".into()))
}

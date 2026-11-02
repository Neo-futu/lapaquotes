//! Extract platform OTC quotes → BI [`QuoteFact`] rows.

use chrono::NaiveDate;
use cornucopia::deadpool_postgres::Pool;
use std::collections::HashMap;

use super::error::BiError;
use super::models::QuoteFact;
use super::store::new_id;
use super::workday::next_hk_workday;
use crate::quote_context::{bond_by_date, deposit_by_date};

pub fn deposit_sync_key(d: NaiveDate, inst: &str, term: &str, ccy: &str) -> String {
    format!("D|{d}|{inst}|{term}|{ccy}")
}

pub fn bond_sync_key(d: NaiveDate, inst: &str, code: &str, ccy: &str) -> String {
    format!("B|{d}|{inst}|{code}|{ccy}")
}

pub fn best_deposit_sync_key(d: NaiveDate, term: &str, ccy: &str) -> String {
    format!("BEST|{d}|{term}|{ccy}")
}

pub async fn extract_day(pool: &Pool, date: NaiveDate) -> Result<Vec<QuoteFact>, BiError> {
    let deposits = deposit_by_date(pool, Some(date), None)
        .await
        .map_err(|e| BiError::Msg(e.to_string()))?;
    let bonds = bond_by_date(pool, Some(date), None)
        .await
        .map_err(|e| BiError::Msg(e.to_string()))?;
    let next_workday = Some(next_hk_workday(pool, date).await?);

    let mut facts = Vec::with_capacity(deposits.len() + bonds.len());
    for d in deposits {
        facts.push(QuoteFact {
            id: new_id(),
            sync_key: deposit_sync_key(d.quote_date, &d.institution_name, &d.term, &d.currency),
            quote_date: d.quote_date,
            next_workday,
            kind: "deposit".into(),
            institution_name: d.institution_name,
            institution_cn_name: d.institution_cn_name,
            term: Some(d.term),
            term_sort: Some(d.term_sort),
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
            next_workday,
            kind: "bond".into(),
            institution_name: b.institution_name,
            institution_cn_name: b.institution_cn_name,
            term: None,
            term_sort: None,
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

/// Same口径 as site「Best Deposit Quote」: highest bid per (term × currency).
/// Rows are Feishu-only (sync_key `BEST|…`); not written to `BiQuoteFact`.
pub fn best_deposit_facts(date: NaiveDate, facts: &[QuoteFact]) -> Vec<QuoteFact> {
    let mut groups: HashMap<(String, String), Vec<&QuoteFact>> = HashMap::new();
    for f in facts {
        if f.kind != "deposit" {
            continue;
        }
        let term = match &f.term {
            Some(t) if !t.is_empty() => t.clone(),
            _ => continue,
        };
        let ccy = match &f.currency {
            Some(c) if !c.is_empty() => c.clone(),
            _ => continue,
        };
        groups.entry((term, ccy)).or_default().push(f);
    }

    let next_workday = facts.first().and_then(|f| f.next_workday);
    let mut out: Vec<QuoteFact> = groups
        .into_iter()
        .filter_map(|((term, currency), rows)| {
            let winner = rows
                .into_iter()
                .filter(|r| r.bid.is_some())
                .max_by(|a, b| a.bid.partial_cmp(&b.bid).unwrap_or(std::cmp::Ordering::Equal))?;
            Some(QuoteFact {
                id: new_id(),
                sync_key: best_deposit_sync_key(date, &term, &currency),
                quote_date: date,
                next_workday,
                kind: "deposit".into(),
                institution_name: winner.institution_name.clone(),
                institution_cn_name: winner.institution_cn_name.clone(),
                term: Some(term),
                term_sort: winner.term_sort,
                currency: Some(currency),
                bond_code: None,
                bond_name: None,
                bid: winner.bid,
                ask: None,
                note: Some("Best Deposit Quote".into()),
                bitable_table_id: None,
                bitable_record_id: None,
                pushed_at: None,
            })
        })
        .collect();

    out.sort_by(|a, b| {
        a.currency
            .cmp(&b.currency)
            .then_with(|| a.term_sort.cmp(&b.term_sort))
            .then_with(|| a.term.cmp(&b.term))
    });
    out
}

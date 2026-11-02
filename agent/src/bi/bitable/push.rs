//! Push BI facts into Feishu ledger / digest tables.

use chrono::{NaiveDate, TimeZone, Utc};
use serde::Serialize;
use serde_json::{json, Map, Value};
use std::collections::{HashMap, HashSet};

use super::client::BitableClient;
use crate::bi::models::QuoteFact;

#[derive(Debug, Clone, Serialize)]
pub struct LedgerPushReport {
    pub created: usize,
    pub updated: usize,
    pub deleted: usize,
    /// sync_key → record_id after push
    pub record_ids: HashMap<String, String>,
}

pub async fn push_ledger(
    client: &BitableClient,
    table_id: &str,
    date: NaiveDate,
    facts: &[QuoteFact],
) -> Result<LedgerPushReport, String> {
    let date_iso = date.to_string();
    let remote = client
        .search_by_sync_contains(table_id, &format!("|{date_iso}|"))
        .await?;

    let mut remote_by_key: HashMap<String, String> = HashMap::new();
    for r in &remote {
        if let Some(k) = &r.sync_key {
            remote_by_key.insert(k.clone(), r.record_id.clone());
        }
    }

    let mut to_create = Vec::new();
    let mut to_update = Vec::new();
    let mut create_keys = Vec::new();
    let desired_keys: HashSet<String> = facts.iter().map(|f| f.sync_key.clone()).collect();

    for f in facts {
        let fields = fact_to_ledger_fields(f);
        if let Some(rid) = remote_by_key.get(&f.sync_key) {
            to_update.push(json!({ "record_id": rid, "fields": fields }));
        } else {
            create_keys.push(f.sync_key.clone());
            to_create.push(json!({ "fields": fields }));
        }
    }

    let stale: Vec<String> = remote
        .into_iter()
        .filter_map(|r| {
            let key = r.sync_key?;
            if desired_keys.contains(&key) {
                None
            } else {
                Some(r.record_id)
            }
        })
        .collect();

    let n_create = to_create.len();
    let created_ids = client.batch_create_with_ids(table_id, to_create).await?;
    let updated = client.batch_update(table_id, to_update).await?;
    let deleted = client.batch_delete(table_id, stale).await?;

    let mut record_ids = remote_by_key;
    for (k, id) in create_keys.into_iter().zip(created_ids.into_iter()) {
        record_ids.insert(k, id);
    }
    // Ensure mapping complete after write.
    if record_ids.len() < facts.len() {
        let remote2 = client
            .search_by_sync_contains(table_id, &format!("|{date_iso}|"))
            .await?;
        for r in remote2 {
            if let Some(k) = r.sync_key {
                record_ids.insert(k, r.record_id);
            }
        }
    }

    Ok(LedgerPushReport {
        created: n_create,
        updated,
        deleted,
        record_ids,
    })
}

fn fact_to_ledger_fields(f: &QuoteFact) -> Value {
    let mut m = Map::new();
    m.insert("同步键".into(), json!(f.sync_key));
    m.insert("日期".into(), json!(date_to_ms(f.quote_date)));
    if let Some(nwd) = f.next_workday {
        m.insert("下一工作日".into(), json!(date_to_ms(nwd)));
    }
    m.insert("机构".into(), json!(f.institution_name));
    if let Some(cn) = &f.institution_cn_name {
        if !cn.is_empty() {
            m.insert("机构中文名".into(), json!(cn));
        }
    }
    m.insert(
        "类型".into(),
        json!(if f.kind == "bond" { "债券" } else { "存款" }),
    );
    if let Some(t) = &f.term {
        m.insert("期限".into(), json!(t));
    }
    if let Some(s) = f.term_sort {
        m.insert("期限排序".into(), json!(s));
    }
    if let Some(c) = &f.currency {
        m.insert("币种".into(), json!(c));
    }
    if let Some(c) = &f.bond_code {
        m.insert("代码".into(), json!(c));
    }
    if let Some(n) = &f.bond_name {
        m.insert("名称".into(), json!(n));
    }
    if let Some(v) = f.bid {
        m.insert("Bid".into(), json!(v));
    }
    if let Some(v) = f.ask {
        m.insert("Ask".into(), json!(v));
    }
    if let Some(n) = &f.note {
        m.insert("备注".into(), json!(n));
    }
    Value::Object(m)
}

pub async fn push_digest(
    client: &BitableClient,
    table_id: &str,
    date: NaiveDate,
    facts: &[QuoteFact],
) -> Result<usize, String> {
    // One digest row per institution+kind, plus a day total.
    let mut groups: HashMap<(String, String), Vec<&QuoteFact>> = HashMap::new();
    for f in facts {
        groups
            .entry((f.institution_name.clone(), f.kind.clone()))
            .or_default()
            .push(f);
    }

    let next_workday_ms = facts
        .first()
        .and_then(|f| f.next_workday)
        .map(date_to_ms);

    let mut desired: HashMap<String, Value> = HashMap::new();
    for ((inst, kind), rows) in &groups {
        let kind_label = if kind == "bond" { "债券" } else { "存款" };
        let sync_key = format!("G|{date}|{inst}|{kind}");
        let summary = rows
            .iter()
            .take(8)
            .map(|r| {
                if kind == "bond" {
                    format!(
                        "{} bid/ask={}/{}",
                        r.bond_code.as_deref().unwrap_or("?"),
                        fmt_opt(r.bid),
                        fmt_opt(r.ask)
                    )
                } else {
                    format!(
                        "{} {} bid/ask={}/{}",
                        r.term.as_deref().unwrap_or("?"),
                        r.currency.as_deref().unwrap_or(""),
                        fmt_opt(r.bid),
                        fmt_opt(r.ask)
                    )
                }
            })
            .collect::<Vec<_>>()
            .join("；");
        let mut fields = json!({
            "同步键": sync_key,
            "机构": inst,
            "类型": kind_label,
            "报价日": date_to_ms(date),
            "条数": rows.len() as i64,
            "摘要": summary,
        });
        if let Some(ms) = next_workday_ms {
            fields
                .as_object_mut()
                .unwrap()
                .insert("下一工作日".into(), json!(ms));
        }
        desired.insert(sync_key.clone(), fields);
    }

    let total_key = format!("G|{date}|__ALL__|all");
    let mut total_fields = json!({
        "同步键": total_key,
        "机构": "全部",
        "类型": "汇总",
        "报价日": date_to_ms(date),
        "条数": facts.len() as i64,
        "摘要": format!("{date} 共 {} 条报价（存款/债券）", facts.len()),
    });
    if let Some(ms) = next_workday_ms {
        total_fields
            .as_object_mut()
            .unwrap()
            .insert("下一工作日".into(), json!(ms));
    }
    desired.insert(total_key.clone(), total_fields);

    let remote = client
        .search_by_sync_contains(table_id, &format!("|{date}|"))
        .await?;
    let mut remote_by_key: HashMap<String, String> = HashMap::new();
    for r in &remote {
        if let Some(k) = &r.sync_key {
            remote_by_key.insert(k.clone(), r.record_id.clone());
        }
    }

    let mut to_create = Vec::new();
    let mut to_update = Vec::new();
    let desired_keys: HashSet<String> = desired.keys().cloned().collect();
    for (key, fields) in desired {
        if let Some(rid) = remote_by_key.get(&key) {
            to_update.push(json!({ "record_id": rid, "fields": fields }));
        } else {
            to_create.push(json!({ "fields": fields }));
        }
    }
    let stale: Vec<String> = remote
        .into_iter()
        .filter_map(|r| {
            let k = r.sync_key?;
            if desired_keys.contains(&k) {
                None
            } else {
                Some(r.record_id)
            }
        })
        .collect();

    let created = client.batch_create(table_id, to_create).await?;
    let updated = client.batch_update(table_id, to_update).await?;
    let _deleted = client.batch_delete(table_id, stale).await?;
    Ok(created + updated)
}

fn fmt_opt(v: Option<f64>) -> String {
    v.map(|x| format!("{x:.4}"))
        .unwrap_or_else(|| "-".into())
}

fn date_to_ms(d: NaiveDate) -> i64 {
    let ndt = d.and_hms_opt(0, 0, 0).expect("midnight");
    Utc.from_utc_datetime(&ndt).timestamp_millis()
}

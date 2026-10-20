//! Discover all tables / views / fields in a Base; assign BI roles.

use tracing::info;

use super::client::BitableClient;
use crate::bi::models::{BitableRole, FieldCatalog, TableCatalog, ViewCatalog};

pub async fn discover_base(client: &BitableClient) -> Result<Vec<TableCatalog>, String> {
    let raw_tables = client.list_tables_raw().await?;
    let mut tables = Vec::with_capacity(raw_tables.len());

    for t in raw_tables {
        let table_id = t
            .get("table_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "table missing table_id".to_string())?
            .to_string();
        let name = t
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let revision = t.get("revision").and_then(|v| v.as_i64()).map(|n| n as i32);

        let views_raw = client.list_views_raw(&table_id).await?;
        let views: Vec<ViewCatalog> = views_raw
            .into_iter()
            .filter_map(|v| {
                Some(ViewCatalog {
                    view_id: v.get("view_id")?.as_str()?.to_string(),
                    name: v.get("view_name")?.as_str()?.to_string(),
                    view_type: v
                        .get("view_type")
                        .and_then(|x| x.as_str())
                        .map(|s| s.to_string()),
                })
            })
            .collect();

        let fields_raw = client.list_fields_raw(&table_id).await?;
        let fields: Vec<FieldCatalog> = fields_raw
            .into_iter()
            .filter_map(|f| {
                Some(FieldCatalog {
                    field_id: f.get("field_id")?.as_str()?.to_string(),
                    name: f.get("field_name")?.as_str()?.to_string(),
                    field_type: f.get("type")?.as_i64()? as i32,
                    ui_type: f
                        .get("ui_type")
                        .and_then(|x| x.as_str())
                        .map(|s| s.to_string()),
                    is_primary: f
                        .get("is_primary")
                        .and_then(|x| x.as_bool())
                        .unwrap_or(false),
                })
            })
            .collect();

        let default_view_id = views.first().map(|v| v.view_id.clone());

        tables.push(TableCatalog {
            table_id,
            name,
            role: BitableRole::Unknown,
            default_view_id,
            revision,
            views,
            fields,
        });
    }

    assign_roles(&mut tables);
    info!(count = tables.len(), "bitable discover_base done");
    Ok(tables)
}

/// Heuristic role assignment for the commodity → OTC template Base.
pub fn assign_roles(tables: &mut [TableCatalog]) {
    // Prefer name hints (order matters: 「最高报价」also contains「报价」).
    for t in tables.iter_mut() {
        let n = t.name.to_lowercase();
        if n.contains("推送") || n.contains("digest") || n.contains("消息") {
            t.role = BitableRole::QuoteDigest;
        } else if n.contains("最高") || n.contains("daily_best") || n.contains("best deposit") {
            t.role = BitableRole::DailyBest;
        } else if n.contains("台账") || n.contains("ledger") || n.contains("报价") {
            t.role = BitableRole::QuoteLedger;
        }
    }

    // Fallback by order: first ledger, second digest.
    let has_ledger = tables.iter().any(|t| t.role == BitableRole::QuoteLedger);
    let has_digest = tables.iter().any(|t| t.role == BitableRole::QuoteDigest);
    if !has_ledger {
        if let Some(t) = tables.first_mut() {
            t.role = BitableRole::QuoteLedger;
        }
    }
    if !has_digest {
        if let Some(t) = tables.iter_mut().find(|t| t.role == BitableRole::Unknown) {
            t.role = BitableRole::QuoteDigest;
        } else if tables.len() > 1 {
            if let Some(t) = tables.get_mut(1) {
                if t.role != BitableRole::QuoteLedger {
                    t.role = BitableRole::QuoteDigest;
                }
            }
        }
    }
}

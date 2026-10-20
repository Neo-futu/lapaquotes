//! Rewrite discovered Base tables into OTC quote schemas.

use serde::Serialize;
use serde_json::{json, Value};
use tracing::info;

use super::client::BitableClient;
use super::discovery::discover_base;
use crate::bi::models::{BitableRole, TableCatalog};

#[derive(Debug, Clone, Serialize)]
pub struct TableMigrateReport {
    pub table_id: String,
    pub role: String,
    pub table_name: String,
    pub deleted_records: usize,
    pub deleted_fields: Vec<String>,
    pub deleted_views: Vec<String>,
    pub keep_view: String,
    pub renamed_primary: Option<String>,
    pub created_fields: Vec<String>,
    pub final_fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MigrateReport {
    pub tables: Vec<TableMigrateReport>,
}

/// Non-destructive: create any missing OTC columns (e.g. 「类型」 if deleted in UI).
pub async fn ensure_otc_fields(client: &BitableClient) -> Result<Vec<String>, String> {
    let catalog = discover_base(client).await?;
    let mut created = Vec::new();
    for t in &catalog {
        let specs = match t.role {
            BitableRole::QuoteLedger | BitableRole::DailyBest => ledger_field_specs(),
            BitableRole::QuoteDigest => digest_field_specs(),
            BitableRole::Unknown => continue,
        };
        let after = client.list_fields_raw(&t.table_id).await?;
        let existing: std::collections::HashSet<String> = after
            .iter()
            .filter_map(|f| f.get("field_name").and_then(|v| v.as_str()).map(|s| s.to_string()))
            .collect();
        for spec in specs {
            if existing.contains(&spec.name) {
                continue;
            }
            client.create_field(&t.table_id, spec.body).await?;
            created.push(format!("{}:{}", t.name, spec.name));
            info!(table = %t.name, field = %spec.name, "restored missing bitable field");
            tokio::time::sleep(std::time::Duration::from_millis(250)).await;
        }
    }
    Ok(created)
}

pub async fn migrate_all_tables(client: &BitableClient) -> Result<MigrateReport, String> {
    let catalog = discover_base(client).await?;
    let mut reports = Vec::new();
    for t in &catalog {
        match t.role {
            BitableRole::QuoteLedger => {
                reports.push(
                    migrate_one(client, t, "OTC报价台账", "全部报价", ledger_field_specs())
                        .await?,
                );
            }
            BitableRole::QuoteDigest => {
                reports.push(
                    migrate_one(client, t, "最新报价推送", "消息内容", digest_field_specs())
                        .await?,
                );
            }
            BitableRole::DailyBest => {
                reports.push(
                    migrate_one(
                        client,
                        t,
                        "OTC每日最高报价",
                        "全部最高",
                        ledger_field_specs(),
                    )
                    .await?,
                );
            }
            BitableRole::Unknown => {
                info!(table = %t.name, "skip unknown-role table");
            }
        }
    }
    Ok(MigrateReport { tables: reports })
}

async fn migrate_one(
    client: &BitableClient,
    table: &TableCatalog,
    target_name: &str,
    keep_view: &str,
    specs: Vec<FieldSpec>,
) -> Result<TableMigrateReport, String> {
    let table_id = &table.table_id;
    let target_names: std::collections::HashSet<&str> =
        specs.iter().map(|s| s.name.as_str()).collect();

    // Clear records.
    let existing = client.list_all_records(table_id).await?;
    let ids: Vec<String> = existing.into_iter().map(|r| r.record_id).collect();
    let deleted_records = client.batch_delete(table_id, ids).await?;

    let _ = client.rename_table(table_id, target_name).await;

    let fields = client.list_fields_raw(table_id).await?;
    let mut deleted_fields = Vec::new();
    let mut renamed_primary = None;
    let mut primary_id = None;
    let mut primary_name = None;

    for f in &fields {
        let id = f.get("field_id").and_then(|v| v.as_str()).unwrap_or("");
        let name = f.get("field_name").and_then(|v| v.as_str()).unwrap_or("");
        let is_primary = f.get("is_primary").and_then(|v| v.as_bool()).unwrap_or(false);
        if is_primary {
            primary_id = Some(id.to_string());
            primary_name = Some(name.to_string());
            continue;
        }
        if id.is_empty() || target_names.contains(name) {
            continue;
        }
        client.delete_field(table_id, id).await?;
        deleted_fields.push(name.to_string());
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    }

    if let Some(pid) = primary_id {
        let cur = primary_name.unwrap_or_default();
        if cur != "同步键" {
            client
                .update_field(
                    table_id,
                    &pid,
                    json!({ "field_name": "同步键", "type": 1, "ui_type": "Text" }),
                )
                .await?;
            renamed_primary = Some(format!("{cur} → 同步键"));
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        }
    }

    let after = client.list_fields_raw(table_id).await?;
    let existing_names: std::collections::HashSet<String> = after
        .iter()
        .filter_map(|f| f.get("field_name").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .collect();

    let mut created_fields = Vec::new();
    for spec in specs {
        if spec.name == "同步键" || existing_names.contains(&spec.name) {
            continue;
        }
        client.create_field(table_id, spec.body).await?;
        created_fields.push(spec.name);
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    }

    let final_fields: Vec<String> = client
        .list_fields_raw(table_id)
        .await?
        .into_iter()
        .filter_map(|f| f.get("field_name").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .collect();

    let deleted_views = client
        .cleanup_views_keep_one(table_id, keep_view)
        .await
        .unwrap_or_default();

    let report = TableMigrateReport {
        table_id: table_id.clone(),
        role: table.role.as_str().into(),
        table_name: target_name.into(),
        deleted_records,
        deleted_fields,
        deleted_views,
        keep_view: keep_view.into(),
        renamed_primary,
        created_fields,
        final_fields,
    };
    info!(?report, "bitable table migrated");
    Ok(report)
}

struct FieldSpec {
    name: String,
    body: Value,
}

fn ledger_field_specs() -> Vec<FieldSpec> {
    vec![
        FieldSpec {
            name: "同步键".into(),
            body: json!({ "field_name": "同步键", "type": 1 }),
        },
        FieldSpec {
            name: "日期".into(),
            body: json!({
                "field_name": "日期",
                "type": 5,
                "ui_type": "DateTime",
                "property": { "date_formatter": "yyyy-MM-dd", "auto_fill": false }
            }),
        },
        FieldSpec {
            name: "机构".into(),
            body: json!({ "field_name": "机构", "type": 1, "ui_type": "Text" }),
        },
        FieldSpec {
            name: "机构中文名".into(),
            body: json!({ "field_name": "机构中文名", "type": 1, "ui_type": "Text" }),
        },
        FieldSpec {
            name: "类型".into(),
            body: json!({
                "field_name": "类型",
                "type": 3,
                "ui_type": "SingleSelect",
                "property": { "options": [{ "name": "存款" }, { "name": "债券" }] }
            }),
        },
        FieldSpec {
            name: "期限".into(),
            body: json!({ "field_name": "期限", "type": 1, "ui_type": "Text" }),
        },
        FieldSpec {
            name: "期限排序".into(),
            body: json!({
                "field_name": "期限排序",
                "type": 2,
                "ui_type": "Number",
                "property": { "formatter": "0" }
            }),
        },
        FieldSpec {
            name: "币种".into(),
            body: json!({ "field_name": "币种", "type": 1, "ui_type": "Text" }),
        },
        FieldSpec {
            name: "Bid".into(),
            body: json!({
                "field_name": "Bid",
                "type": 2,
                "ui_type": "Number",
                "property": { "formatter": "0.0000" }
            }),
        },
        FieldSpec {
            name: "Ask".into(),
            body: json!({
                "field_name": "Ask",
                "type": 2,
                "ui_type": "Number",
                "property": { "formatter": "0.0000" }
            }),
        },
        FieldSpec {
            name: "代码".into(),
            body: json!({ "field_name": "代码", "type": 1, "ui_type": "Text" }),
        },
        FieldSpec {
            name: "名称".into(),
            body: json!({ "field_name": "名称", "type": 1, "ui_type": "Text" }),
        },
        FieldSpec {
            name: "备注".into(),
            body: json!({ "field_name": "备注", "type": 1, "ui_type": "Text" }),
        },
    ]
}

fn digest_field_specs() -> Vec<FieldSpec> {
    vec![
        FieldSpec {
            name: "同步键".into(),
            body: json!({ "field_name": "同步键", "type": 1 }),
        },
        FieldSpec {
            name: "机构".into(),
            body: json!({ "field_name": "机构", "type": 1, "ui_type": "Text" }),
        },
        FieldSpec {
            name: "类型".into(),
            body: json!({
                "field_name": "类型",
                "type": 3,
                "ui_type": "SingleSelect",
                "property": { "options": [{ "name": "存款" }, { "name": "债券" }, { "name": "汇总" }] }
            }),
        },
        FieldSpec {
            name: "报价日".into(),
            body: json!({
                "field_name": "报价日",
                "type": 5,
                "ui_type": "DateTime",
                "property": { "date_formatter": "yyyy-MM-dd", "auto_fill": false }
            }),
        },
        FieldSpec {
            name: "条数".into(),
            body: json!({
                "field_name": "条数",
                "type": 2,
                "ui_type": "Number",
                "property": { "formatter": "0" }
            }),
        },
        FieldSpec {
            name: "摘要".into(),
            body: json!({ "field_name": "摘要", "type": 1, "ui_type": "Text" }),
        },
    ]
}

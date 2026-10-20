//! Feishu Base dashboards (图表仪表盘) + ledger filtered views.
//!
//! Official open API with personalBaseToken supports:
//! - list: `GET .../dashboards`
//! - copy: `POST .../dashboards/{block_id}/copy`
//! - view create / PATCH filter (`property.filter_info`)
//!
//! Chart block create/update is **not** available on `base-api` with `pt-` tokens.
//! Dashboard equality filters therefore rely on **table views** as chart data sources.
//!
//! View filter rules (Feishu PATCH view):
//! - use `field_id` (not field_name)
//! - `value` is a **JSON string**, e.g. `"[\"optXxx\"]"` / `"[\"1M\"]"`
//! - SingleSelect must use **option id**, not option display name

use chrono::{NaiveDate, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{info, warn};

use super::client::BitableClient;

/// Legacy commodity dashboard from the template Base.
pub const DEFAULT_SOURCE_DASHBOARD_ID: &str = "blkRgeGoMaQuEZaB";
pub const DEFAULT_OTC_DASHBOARD_NAME: &str = "OTC报价仪表盘";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardInfo {
    pub block_id: String,
    pub name: String,
    pub role: String,
    pub url: String,
    pub source_block_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DashboardEnsureReport {
    pub source: DashboardInfo,
    pub otc: DashboardInfo,
    pub created: bool,
    pub note: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct LedgerViewsReport {
    pub ensured: Vec<String>,
    pub updated_filters: Vec<String>,
}

struct FieldIndex {
    by_name: std::collections::HashMap<String, Value>,
}

impl FieldIndex {
    async fn load(client: &BitableClient, table_id: &str) -> Result<Self, String> {
        let fields = client.list_fields_raw(table_id).await?;
        let mut by_name = std::collections::HashMap::new();
        for f in fields {
            if let Some(n) = f.get("field_name").and_then(|x| x.as_str()) {
                by_name.insert(n.to_string(), f);
            }
        }
        Ok(Self { by_name })
    }

    fn id(&self, name: &str) -> Result<&str, String> {
        self.by_name
            .get(name)
            .and_then(|f| f.get("field_id"))
            .and_then(|x| x.as_str())
            .ok_or_else(|| format!("field not found: {name}"))
    }

    fn option_id(&self, field_name: &str, option_name: &str) -> Result<String, String> {
        let f = self
            .by_name
            .get(field_name)
            .ok_or_else(|| format!("field not found: {field_name}"))?;
        let opts = f
            .get("property")
            .and_then(|p| p.get("options"))
            .and_then(|o| o.as_array())
            .ok_or_else(|| format!("field {field_name} has no options"))?;
        for o in opts {
            if o.get("name").and_then(|x| x.as_str()) == Some(option_name) {
                if let Some(id) = o.get("id").and_then(|x| x.as_str()) {
                    return Ok(id.to_string());
                }
            }
        }
        Err(format!("option {option_name:?} not found on {field_name}"))
    }
}

/// One equality filter condition for PATCH view.
#[derive(Debug, Clone)]
enum EqFilter {
    /// SingleSelect by option display name → resolved to option id
    Select { field: &'static str, option: &'static str },
    /// Text / number equality
    Text { field: &'static str, value: String },
    /// Date = ExactDate(ms)
    ExactDate { field: &'static str, date: NaiveDate },
}

fn filter_value_json(parts: &[String]) -> String {
    serde_json::to_string(parts).unwrap_or_else(|_| "[]".into())
}

fn build_conditions(idx: &FieldIndex, filters: &[EqFilter]) -> Result<Vec<Value>, String> {
    let mut out = Vec::with_capacity(filters.len());
    for f in filters {
        let (field_id, value) = match f {
            EqFilter::Select { field, option } => {
                let fid = idx.id(field)?;
                let opt = idx.option_id(field, option)?;
                (fid.to_string(), filter_value_json(&[opt]))
            }
            EqFilter::Text { field, value } => {
                let fid = idx.id(field)?;
                (fid.to_string(), filter_value_json(&[value.clone()]))
            }
            EqFilter::ExactDate { field, date } => {
                let fid = idx.id(field)?;
                let ms = date_to_ms(*date).to_string();
                (
                    fid.to_string(),
                    filter_value_json(&["ExactDate".into(), ms]),
                )
            }
        };
        out.push(json!({
            "field_id": field_id,
            "operator": "is",
            "value": value,
        }));
    }
    Ok(out)
}

async fn ensure_named_view(
    client: &BitableClient,
    table_id: &str,
    name: &str,
    existing: &std::collections::HashMap<String, String>,
) -> Result<(String, bool), String> {
    if let Some(id) = existing.get(name) {
        return Ok((id.clone(), false));
    }
    let id = client.create_view(table_id, name).await?;
    tokio::time::sleep(std::time::Duration::from_millis(250)).await;
    Ok((id, true))
}

async fn apply_view_filters(
    client: &BitableClient,
    table_id: &str,
    view_id: &str,
    idx: &FieldIndex,
    filters: &[EqFilter],
    sort_by_term: bool,
) -> Result<(), String> {
    let conditions = build_conditions(idx, filters)?;
    let mut property = json!({
        "filter_info": {
            "conjunction": "and",
            "conditions": conditions
        }
    });
    if sort_by_term {
        if let Ok(fid) = idx.id("期限排序") {
            property["sort_info"] = json!([{ "field_id": fid, "desc": false }]);
        }
    }
    client
        .patch_view(table_id, view_id, json!({ "property": property }))
        .await
}

pub async fn list_dashboards(client: &BitableClient) -> Result<Vec<DashboardInfo>, String> {
    let url = format!(
        "{}/open-apis/bitable/v1/apps/{}/dashboards?page_size=100",
        client.cfg.api_base, client.cfg.app_token
    );
    let data = client.get(&url).await?;
    let items = data
        .get("dashboards")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    Ok(items
        .into_iter()
        .filter_map(|d| {
            let block_id = d.get("block_id")?.as_str()?.to_string();
            let name = d
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let role = classify_dashboard(&block_id, &name);
            Some(DashboardInfo {
                url: dashboard_url(&client.cfg.app_token, &block_id),
                block_id,
                name,
                role: role.to_string(),
                source_block_id: None,
            })
        })
        .collect())
}

pub async fn copy_dashboard(
    client: &BitableClient,
    source_block_id: &str,
    name: &str,
) -> Result<DashboardInfo, String> {
    let url = format!(
        "{}/open-apis/bitable/v1/apps/{}/dashboards/{}/copy",
        client.cfg.api_base, client.cfg.app_token, source_block_id
    );
    let data = client.post(&url, json!({ "name": name })).await?;
    let block_id = data
        .get("block_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("copy dashboard missing block_id: {data}"))?
        .to_string();
    let name = data
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or(name)
        .to_string();
    Ok(DashboardInfo {
        url: dashboard_url(&client.cfg.app_token, &block_id),
        block_id,
        name,
        role: "quote_dashboard".into(),
        source_block_id: Some(source_block_id.to_string()),
    })
}

/// Ensure an OTC-named dashboard exists (copy from template if needed).
pub async fn ensure_otc_dashboard(client: &BitableClient) -> Result<DashboardEnsureReport, String> {
    let all = list_dashboards(client).await?;
    let source = all
        .iter()
        .find(|d| d.block_id == DEFAULT_SOURCE_DASHBOARD_ID)
        .cloned()
        .or_else(|| {
            all.iter()
                .find(|d| d.role == "legacy_commodity")
                .cloned()
        })
        .or_else(|| all.first().cloned())
        .ok_or_else(|| "no dashboard found in Base".to_string())?;

    if let Some(existing) = all.iter().find(|d| {
        d.role == "quote_dashboard" || d.name.contains("OTC") || d.name.contains("报价仪表")
    }) {
        info!(block_id = %existing.block_id, "otc dashboard already present");
        return Ok(DashboardEnsureReport {
            source: source.clone(),
            otc: existing.clone(),
            created: false,
            note: "仪表盘图表请选台账「筛选视图」作数据源（如「存款报价」「1M USD 存款跨机构」）；pt- 无法改图表块。详见 doc/bi.md".into(),
        });
    }

    let otc = copy_dashboard(client, &source.block_id, DEFAULT_OTC_DASHBOARD_NAME).await?;
    info!(block_id = %otc.block_id, "copied otc dashboard");
    Ok(DashboardEnsureReport {
        source,
        otc,
        created: true,
        note: "已复制仪表盘壳；请在 UI 将图表数据源绑到 OTC 台账的筛选视图。".into(),
    })
}

/// Create / refresh chart-oriented filtered grid views on the ledger (idempotent by name).
///
/// When `quote_date` is set, also refreshes「最新日*」views to that ExactDate.
pub async fn ensure_ledger_chart_views(
    client: &BitableClient,
    table_id: &str,
) -> Result<LedgerViewsReport, String> {
    ensure_ledger_chart_views_for_date(client, table_id, None).await
}

pub async fn ensure_ledger_chart_views_for_date(
    client: &BitableClient,
    table_id: &str,
    quote_date: Option<NaiveDate>,
) -> Result<LedgerViewsReport, String> {
    let idx = FieldIndex::load(client, table_id).await?;
    let existing_raw = client.list_views_raw(table_id).await?;
    let mut existing: std::collections::HashMap<String, String> = existing_raw
        .iter()
        .filter_map(|v| {
            Some((
                v.get("view_name")?.as_str()?.to_string(),
                v.get("view_id")?.as_str()?.to_string(),
            ))
        })
        .collect();

    // Static equality views for dashboards / comparisons.
    let mut specs: Vec<(&str, Vec<EqFilter>)> = vec![
        ("全部报价", vec![]),
        (
            "存款报价",
            vec![EqFilter::Select {
                field: "类型",
                option: "存款",
            }],
        ),
        (
            "债券报价",
            vec![EqFilter::Select {
                field: "类型",
                option: "债券",
            }],
        ),
        (
            "1M USD 存款跨机构",
            vec![
                EqFilter::Select {
                    field: "类型",
                    option: "存款",
                },
                EqFilter::Text {
                    field: "期限",
                    value: "1M".into(),
                },
                EqFilter::Text {
                    field: "币种",
                    value: "USD".into(),
                },
            ],
        ),
        (
            "ABC HK 美元存款各期限",
            vec![
                EqFilter::Select {
                    field: "类型",
                    option: "存款",
                },
                EqFilter::Text {
                    field: "机构",
                    value: "Agricultural Bank of China Hong Kong".into(),
                },
                EqFilter::Text {
                    field: "币种",
                    value: "USD".into(),
                },
            ],
        ),
        (
            "USD 存款",
            vec![
                EqFilter::Select {
                    field: "类型",
                    option: "存款",
                },
                EqFilter::Text {
                    field: "币种",
                    value: "USD".into(),
                },
            ],
        ),
        (
            "CNY 存款",
            vec![
                EqFilter::Select {
                    field: "类型",
                    option: "存款",
                },
                EqFilter::Text {
                    field: "币种",
                    value: "CNY".into(),
                },
            ],
        ),
    ];

    if let Some(d) = quote_date {
        specs.push((
            "最新日存款",
            vec![
                EqFilter::Select {
                    field: "类型",
                    option: "存款",
                },
                EqFilter::ExactDate {
                    field: "日期",
                    date: d,
                },
            ],
        ));
        specs.push((
            "最新日债券",
            vec![
                EqFilter::Select {
                    field: "类型",
                    option: "债券",
                },
                EqFilter::ExactDate {
                    field: "日期",
                    date: d,
                },
            ],
        ));
        // Radar / multi-term cross-institution (USD Bid).
        specs.push((
            "最新日 USD 存款",
            vec![
                EqFilter::Select {
                    field: "类型",
                    option: "存款",
                },
                EqFilter::Text {
                    field: "币种",
                    value: "USD".into(),
                },
                EqFilter::ExactDate {
                    field: "日期",
                    date: d,
                },
            ],
        ));
        specs.push((
            "最新日 ABC HK 美元存款",
            vec![
                EqFilter::Select {
                    field: "类型",
                    option: "存款",
                },
                EqFilter::Text {
                    field: "机构",
                    value: "Agricultural Bank of China Hong Kong".into(),
                },
                EqFilter::Text {
                    field: "币种",
                    value: "USD".into(),
                },
                EqFilter::ExactDate {
                    field: "日期",
                    date: d,
                },
            ],
        ));
        specs.push((
            "最新日 1M USD 存款",
            vec![
                EqFilter::Select {
                    field: "类型",
                    option: "存款",
                },
                EqFilter::Text {
                    field: "期限",
                    value: "1M".into(),
                },
                EqFilter::Text {
                    field: "币种",
                    value: "USD".into(),
                },
                EqFilter::ExactDate {
                    field: "日期",
                    date: d,
                },
            ],
        ));
    }

    let mut ensured = Vec::new();
    let mut updated_filters = Vec::new();

    for (name, filters) in specs {
        let (view_id, created) =
            ensure_named_view(client, table_id, name, &existing).await?;
        if created {
            ensured.push(format!("{name}:{view_id}"));
            existing.insert(name.to_string(), view_id.clone());
        }
        if filters.is_empty() {
            continue;
        }
        // Deposit views: sort by numeric 期限排序 so O/N→1W→1M… (text「期限」 cannot).
        let sort_by_term = name.contains("存款");
        match apply_view_filters(client, table_id, &view_id, &idx, &filters, sort_by_term)
            .await
        {
            Ok(()) => {
                updated_filters.push(name.to_string());
                info!(view = name, %view_id, "ledger view filters applied");
            }
            Err(e) => {
                warn!(view = name, error = %e, "failed to apply view filters");
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
    }

    Ok(LedgerViewsReport {
        ensured,
        updated_filters,
    })
}

fn classify_dashboard(block_id: &str, name: &str) -> &'static str {
    if block_id == DEFAULT_SOURCE_DASHBOARD_ID
        || name.contains("铜")
        || name.contains("铝")
        || name.contains("塑钢")
        || name.contains("密封胶")
    {
        "legacy_commodity"
    } else if name.contains("OTC") || name.contains("报价仪表") {
        "quote_dashboard"
    } else {
        "unknown"
    }
}

fn dashboard_url(app_token: &str, block_id: &str) -> String {
    let host = std::env::var("FEISHU_BITABLE_WEB_HOST")
        .or_else(|_| std::env::var("FEISHU_WEB_HOST"))
        .unwrap_or_else(|_| "https://futu.feishu.cn".into())
        .trim_end_matches('/')
        .to_string();
    format!("{host}/base/{app_token}?table={block_id}")
}

fn date_to_ms(d: NaiveDate) -> i64 {
    let ndt = d.and_hms_opt(0, 0, 0).expect("midnight");
    Utc.from_utc_datetime(&ndt).timestamp_millis()
}

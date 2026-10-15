//! Low-level Feishu Bitable HTTP (personalBaseToken / open platform).

use serde::Deserialize;
use serde_json::{Value, json};
use tracing::info;

use super::config::BitableConfig;

#[derive(Debug, Clone)]
pub struct BitableClient {
    pub cfg: BitableConfig,
    http: reqwest::Client,
}

#[derive(Debug, Clone)]
pub struct RemoteRecord {
    pub record_id: String,
    pub sync_key: Option<String>,
    pub fields: Value,
}

#[derive(Debug, Deserialize)]
struct ApiEnvelope {
    code: i64,
    #[serde(default)]
    msg: String,
    #[serde(default)]
    data: Option<Value>,
}

impl BitableClient {
    pub fn connect() -> Result<Self, String> {
        let cfg = BitableConfig::from_env()?;
        info!(
            app_token = %cfg.app_token,
            api_base = %cfg.api_base,
            "bitable client ready"
        );
        Ok(Self {
            cfg,
            http: reqwest::Client::new(),
        })
    }

    async fn parse(resp: reqwest::Response) -> Result<Value, String> {
        let status = resp.status();
        let text = resp.text().await.map_err(|e| e.to_string())?;
        if !status.is_success() {
            return Err(format!("bitable HTTP {status}: {text}"));
        }
        let env: ApiEnvelope =
            serde_json::from_str(&text).map_err(|e| format!("bitable json: {e}; {text}"))?;
        if env.code != 0 {
            return Err(format!("bitable code={}: {} — {text}", env.code, env.msg));
        }
        Ok(env.data.unwrap_or(Value::Null))
    }

    pub async fn get(&self, url: &str) -> Result<Value, String> {
        let resp = self
            .http
            .get(url)
            .bearer_auth(&self.cfg.access_token)
            .send()
            .await
            .map_err(|e| format!("GET {url}: {e}"))?;
        Self::parse(resp).await
    }

    pub async fn post(&self, url: &str, body: Value) -> Result<Value, String> {
        let resp = self
            .http
            .post(url)
            .bearer_auth(&self.cfg.access_token)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("POST {url}: {e}"))?;
        Self::parse(resp).await
    }

    pub async fn put(&self, url: &str, body: Value) -> Result<Value, String> {
        let resp = self
            .http
            .put(url)
            .bearer_auth(&self.cfg.access_token)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("PUT {url}: {e}"))?;
        Self::parse(resp).await
    }

    pub async fn patch(&self, url: &str, body: Value) -> Result<Value, String> {
        let resp = self
            .http
            .patch(url)
            .bearer_auth(&self.cfg.access_token)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("PATCH {url}: {e}"))?;
        Self::parse(resp).await
    }

    pub async fn delete(&self, url: &str) -> Result<Value, String> {
        let resp = self
            .http
            .delete(url)
            .bearer_auth(&self.cfg.access_token)
            .send()
            .await
            .map_err(|e| format!("DELETE {url}: {e}"))?;
        Self::parse(resp).await
    }

    pub async fn list_tables_raw(&self) -> Result<Vec<Value>, String> {
        paginate_get(self, &format!("{}?page_size=100", self.cfg.tables_url())).await
    }

    pub async fn list_fields_raw(&self, table_id: &str) -> Result<Vec<Value>, String> {
        paginate_get(
            self,
            &format!("{}?page_size=100", self.cfg.fields_url(table_id)),
        )
        .await
    }

    pub async fn list_views_raw(&self, table_id: &str) -> Result<Vec<Value>, String> {
        paginate_get(
            self,
            &format!("{}?page_size=100", self.cfg.views_url(table_id)),
        )
        .await
    }

    pub async fn rename_table(&self, table_id: &str, name: &str) -> Result<(), String> {
        self.patch(&self.cfg.table_url(table_id), json!({ "name": name }))
            .await?;
        Ok(())
    }

    pub async fn create_view(&self, table_id: &str, view_name: &str) -> Result<String, String> {
        let data = self
            .post(
                &self.cfg.views_url(table_id),
                json!({ "view_name": view_name, "view_type": "grid" }),
            )
            .await?;
        let id = data
            .get("view")
            .and_then(|v| v.get("view_id"))
            .or_else(|| data.get("view_id"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| format!("create_view missing view_id: {data}"))?
            .to_string();
        Ok(id)
    }

    pub async fn patch_view(
        &self,
        table_id: &str,
        view_id: &str,
        body: Value,
    ) -> Result<(), String> {
        let url = format!("{}/{}", self.cfg.views_url(table_id), view_id);
        self.patch(&url, body).await?;
        Ok(())
    }

    pub async fn delete_view(&self, table_id: &str, view_id: &str) -> Result<(), String> {
        let url = format!("{}/{}", self.cfg.views_url(table_id), view_id);
        self.delete(&url).await?;
        Ok(())
    }

    /// Keep a single named grid view; delete all other views.
    pub async fn cleanup_views_keep_one(
        &self,
        table_id: &str,
        keep_name: &str,
    ) -> Result<Vec<String>, String> {
        let mut views = self.list_views_raw(table_id).await?;
        let mut keep_id = views.iter().find_map(|v| {
            let name = v.get("view_name").and_then(|x| x.as_str())?;
            if name == keep_name {
                v.get("view_id").and_then(|x| x.as_str()).map(|s| s.to_string())
            } else {
                None
            }
        });
        if keep_id.is_none() {
            keep_id = Some(self.create_view(table_id, keep_name).await?);
            tokio::time::sleep(std::time::Duration::from_millis(250)).await;
            views = self.list_views_raw(table_id).await?;
        }
        let keep_id = keep_id.expect("keep view");
        let mut deleted = Vec::new();
        for v in &views {
            let vid = v.get("view_id").and_then(|x| x.as_str()).unwrap_or("");
            let vname = v.get("view_name").and_then(|x| x.as_str()).unwrap_or("");
            if vid.is_empty() || vid == keep_id {
                continue;
            }
            self.delete_view(table_id, vid).await?;
            deleted.push(vname.to_string());
            tokio::time::sleep(std::time::Duration::from_millis(250)).await;
        }
        // Ensure keep view has the intended name.
        self.patch_view(table_id, &keep_id, json!({ "view_name": keep_name }))
            .await
            .ok();
        Ok(deleted)
    }

    pub async fn create_field(&self, table_id: &str, body: Value) -> Result<Value, String> {
        let data = self.post(&self.cfg.fields_url(table_id), body).await?;
        Ok(data.get("field").cloned().unwrap_or(data))
    }

    pub async fn update_field(
        &self,
        table_id: &str,
        field_id: &str,
        body: Value,
    ) -> Result<Value, String> {
        let url = format!("{}/{}", self.cfg.fields_url(table_id), field_id);
        let data = self.put(&url, body).await?;
        Ok(data.get("field").cloned().unwrap_or(data))
    }

    pub async fn delete_field(&self, table_id: &str, field_id: &str) -> Result<(), String> {
        let url = format!("{}/{}", self.cfg.fields_url(table_id), field_id);
        self.delete(&url).await?;
        Ok(())
    }

    pub async fn search_records(
        &self,
        table_id: &str,
        body: Value,
    ) -> Result<(Vec<RemoteRecord>, Option<String>, bool), String> {
        let url = format!("{}/search", self.cfg.records_url(table_id));
        let data = self.post(&url, body).await?;
        let sync_field = "同步键";
        let mut out = Vec::new();
        collect_records(&data, sync_field, &mut out);
        let has_more = data
            .get("has_more")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let page_token = data
            .get("page_token")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        Ok((out, page_token, has_more))
    }

    pub async fn list_all_records(&self, table_id: &str) -> Result<Vec<RemoteRecord>, String> {
        let mut out = Vec::new();
        let mut page_token: Option<String> = None;
        loop {
            let mut body = json!({ "page_size": 500 });
            if let Some(pt) = &page_token {
                body["page_token"] = json!(pt);
            }
            let (chunk, next, more) = self.search_records(table_id, body).await?;
            out.extend(chunk);
            if !more {
                break;
            }
            page_token = next;
        }
        Ok(out)
    }

    pub async fn search_by_sync_contains(
        &self,
        table_id: &str,
        needle: &str,
    ) -> Result<Vec<RemoteRecord>, String> {
        let mut out = Vec::new();
        let mut page_token: Option<String> = None;
        loop {
            let mut body = json!({
                "page_size": 500,
                "filter": {
                    "conjunction": "and",
                    "conditions": [{
                        "field_name": "同步键",
                        "operator": "contains",
                        "value": [needle]
                    }]
                }
            });
            if let Some(pt) = &page_token {
                body["page_token"] = json!(pt);
            }
            let (chunk, next, more) = self.search_records(table_id, body).await?;
            out.extend(chunk);
            if !more {
                break;
            }
            page_token = next;
        }
        Ok(out)
    }

    pub async fn batch_create(&self, table_id: &str, records: Vec<Value>) -> Result<usize, String> {
        if records.is_empty() {
            return Ok(0);
        }
        let url = format!("{}/batch_create", self.cfg.records_url(table_id));
        let mut total = 0;
        for chunk in records.chunks(500) {
            let data = self.post(&url, json!({ "records": chunk })).await?;
            total += data
                .get("records")
                .and_then(|v| v.as_array())
                .map(|a| a.len())
                .unwrap_or(chunk.len());
        }
        Ok(total)
    }

    pub async fn batch_update(&self, table_id: &str, records: Vec<Value>) -> Result<usize, String> {
        if records.is_empty() {
            return Ok(0);
        }
        let url = format!("{}/batch_update", self.cfg.records_url(table_id));
        let mut total = 0;
        for chunk in records.chunks(500) {
            let data = self.post(&url, json!({ "records": chunk })).await?;
            total += data
                .get("records")
                .and_then(|v| v.as_array())
                .map(|a| a.len())
                .unwrap_or(chunk.len());
        }
        Ok(total)
    }

    pub async fn batch_delete(&self, table_id: &str, record_ids: Vec<String>) -> Result<usize, String> {
        if record_ids.is_empty() {
            return Ok(0);
        }
        let url = format!("{}/batch_delete", self.cfg.records_url(table_id));
        let mut total = 0;
        for chunk in record_ids.chunks(500) {
            self.post(&url, json!({ "records": chunk })).await?;
            total += chunk.len();
        }
        Ok(total)
    }

    /// batch_create and return record_ids in order (best-effort).
    pub async fn batch_create_with_ids(
        &self,
        table_id: &str,
        records: Vec<Value>,
    ) -> Result<Vec<String>, String> {
        if records.is_empty() {
            return Ok(vec![]);
        }
        let url = format!("{}/batch_create", self.cfg.records_url(table_id));
        let mut ids = Vec::new();
        for chunk in records.chunks(500) {
            let data = self.post(&url, json!({ "records": chunk })).await?;
            if let Some(arr) = data.get("records").and_then(|v| v.as_array()) {
                for r in arr {
                    if let Some(id) = r.get("record_id").and_then(|v| v.as_str()) {
                        ids.push(id.to_string());
                    }
                }
            }
        }
        Ok(ids)
    }
}

async fn paginate_get(client: &BitableClient, first_url: &str) -> Result<Vec<Value>, String> {
    let mut out = Vec::new();
    let mut url = first_url.to_string();
    loop {
        let data = client.get(&url).await?;
        if let Some(items) = data.get("items").and_then(|v| v.as_array()) {
            out.extend(items.iter().cloned());
        }
        let has_more = data
            .get("has_more")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if !has_more {
            break;
        }
        let Some(pt) = data.get("page_token").and_then(|v| v.as_str()) else {
            break;
        };
        let base = first_url.split('?').next().unwrap_or(first_url);
        url = format!("{base}?page_size=100&page_token={pt}");
    }
    Ok(out)
}

fn collect_records(data: &Value, sync_field: &str, out: &mut Vec<RemoteRecord>) {
    let items = data
        .get("items")
        .or_else(|| data.get("records"))
        .and_then(|v| v.as_array());
    let Some(items) = items else {
        return;
    };
    for item in items {
        let record_id = item
            .get("record_id")
            .or_else(|| item.get("id"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if record_id.is_empty() {
            continue;
        }
        let fields = item.get("fields").cloned().unwrap_or(Value::Null);
        let sync_key = field_text(&fields, sync_field);
        out.push(RemoteRecord {
            record_id,
            sync_key,
            fields,
        });
    }
}

fn field_text(fields: &Value, name: &str) -> Option<String> {
    let v = fields.get(name)?;
    match v {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Array(a) => a
            .first()
            .and_then(|x| x.get("text").and_then(|t| t.as_str()))
            .map(|s| s.to_string())
            .or_else(|| a.first().and_then(|x| x.as_str()).map(|s| s.to_string())),
        _ => v.as_str().map(|s| s.to_string()),
    }
}

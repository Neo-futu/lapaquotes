//! Minimal Feishu Bitable credentials (no hard-coded table ids).

#[derive(Debug, Clone)]
pub struct BitableConfig {
    pub access_token: String,
    pub app_token: String,
    pub api_base: String,
}

impl BitableConfig {
    pub fn from_env() -> Result<Self, String> {
        let access_token = first_env(&[
            "FEISHU_BITABLE_ACCESS_TOKEN",
            "FEISHU_PERSONAL_BASE_TOKEN",
        ])
        .ok_or_else(|| {
            "FEISHU_BITABLE_ACCESS_TOKEN (pt-…) required".to_string()
        })?;

        let app_token = first_env(&["FEISHU_BITABLE_APP_TOKEN"]).ok_or_else(|| {
            "FEISHU_BITABLE_APP_TOKEN required".to_string()
        })?;

        let default_base = if access_token.starts_with("pt-") {
            "https://base-api.feishu.cn"
        } else {
            "https://open.feishu.cn"
        };
        let api_base = first_env(&["FEISHU_BITABLE_API_BASE", "FEISHU_API_BASE"])
            .unwrap_or_else(|| default_base.into())
            .trim_end_matches('/')
            .to_string();

        Ok(Self {
            access_token,
            app_token,
            api_base,
        })
    }

    pub fn tables_url(&self) -> String {
        format!(
            "{}/open-apis/bitable/v1/apps/{}/tables",
            self.api_base, self.app_token
        )
    }

    pub fn table_url(&self, table_id: &str) -> String {
        format!("{}/{}", self.tables_url(), table_id)
    }

    pub fn fields_url(&self, table_id: &str) -> String {
        format!("{}/fields", self.table_url(table_id))
    }

    pub fn views_url(&self, table_id: &str) -> String {
        format!("{}/views", self.table_url(table_id))
    }

    pub fn records_url(&self, table_id: &str) -> String {
        format!("{}/records", self.table_url(table_id))
    }
}

fn first_env(keys: &[&str]) -> Option<String> {
    for key in keys {
        if let Ok(v) = std::env::var(key) {
            let t = v.trim();
            if !t.is_empty() {
                return Some(t.to_string());
            }
        }
    }
    None
}

//! BI orchestration service.

use chrono::NaiveDate;
use cornucopia::deadpool_postgres::Pool;
use serde_json::json;
use tracing::info;

use super::bitable::{self, BitableClient};
use super::error::BiError;
use super::extract::{best_deposit_facts, extract_day, resolve_quote_date};
use super::models::{BitableRole, SyncDayReport, TableCatalog, VerifyReport};
use super::store::BiStore;

pub struct BiService {
    store: BiStore,
    pool: Pool,
}

impl BiService {
    pub fn new(pool: Pool) -> Self {
        Self {
            store: BiStore::new(pool.clone()),
            pool,
        }
    }

    pub async fn refresh_catalog(&self) -> Result<Vec<TableCatalog>, BiError> {
        let job = self.store.jobs().start("catalog", None).await?;
        let result: Result<Vec<TableCatalog>, BiError> = async {
            let client = BitableClient::connect().map_err(BiError::bitable)?;
            let tables = bitable::discover_base(&client)
                .await
                .map_err(BiError::bitable)?;
            self.store
                .catalog()
                .replace_catalog(&client.cfg.app_token, &tables)
                .await?;
            let dashboards = bitable::list_dashboards(&client)
                .await
                .map_err(BiError::bitable)?;
            self.store
                .catalog()
                .replace_dashboards(&client.cfg.app_token, &dashboards)
                .await?;
            Ok(tables)
        }
        .await;

        match &result {
            Ok(tables) => {
                let _ = self
                    .store
                    .jobs()
                    .finish_ok(
                        &job,
                        0,
                        0,
                        0,
                        tables.len() as i32,
                        json!({ "tables": tables.len() }),
                    )
                    .await;
            }
            Err(e) => {
                let _ = self.store.jobs().finish_err(&job, &e.to_string()).await;
            }
        }
        result
    }

    pub async fn migrate_bitable(&self) -> Result<bitable::MigrateReport, BiError> {
        let job = self.store.jobs().start("migrate", None).await?;
        let result: Result<bitable::MigrateReport, BiError> = async {
            let client = BitableClient::connect().map_err(BiError::bitable)?;
            let report = bitable::migrate_all_tables(&client)
                .await
                .map_err(BiError::bitable)?;
            // Chart-oriented views + OTC dashboard shell.
            let tables = bitable::discover_base(&client)
                .await
                .map_err(BiError::bitable)?;
            if let Some(ledger) = tables.iter().find(|t| t.role == BitableRole::QuoteLedger) {
                let _ = bitable::ensure_ledger_chart_views_for_date(
                    &client,
                    &ledger.table_id,
                    None,
                )
                .await;
            }
            let _ = bitable::ensure_otc_dashboard(&client).await;
            self.store
                .catalog()
                .replace_catalog(&client.cfg.app_token, &tables)
                .await?;
            let dashboards = bitable::list_dashboards(&client)
                .await
                .map_err(BiError::bitable)?;
            self.store
                .catalog()
                .replace_dashboards(&client.cfg.app_token, &dashboards)
                .await?;
            Ok(report)
        }
        .await;

        match &result {
            Ok(r) => {
                let _ = self
                    .store
                    .jobs()
                    .finish_ok(
                        &job,
                        0,
                        0,
                        0,
                        r.tables.len() as i32,
                        serde_json::to_value(r).unwrap_or(json!({})),
                    )
                    .await;
            }
            Err(e) => {
                let _ = self.store.jobs().finish_err(&job, &e.to_string()).await;
            }
        }
        result
    }

    pub async fn sync_day(&self, date: Option<NaiveDate>) -> Result<SyncDayReport, BiError> {
        let date = resolve_quote_date(&self.pool, date).await?;
        let job = self.store.jobs().start("sync_day", Some(date)).await?;

        let result: Result<SyncDayReport, BiError> = async {
            // Ensure catalog exists; refresh when ledger/best roles missing
            // (e.g. 「每日最高报价」was previously mis-tagged as quote_ledger).
            let mut ledger = self
                .store
                .catalog()
                .table_by_role(BitableRole::QuoteLedger)
                .await?;
            let mut daily_best = self
                .store
                .catalog()
                .table_by_role(BitableRole::DailyBest)
                .await?;
            if ledger.is_none() || daily_best.is_none() {
                self.refresh_catalog().await?;
                ledger = self
                    .store
                    .catalog()
                    .table_by_role(BitableRole::QuoteLedger)
                    .await?;
                daily_best = self
                    .store
                    .catalog()
                    .table_by_role(BitableRole::DailyBest)
                    .await?;
            }
            let ledger = ledger.ok_or_else(|| {
                BiError::Msg("no quote_ledger table in catalog; run migrate".into())
            })?;
            let digest = self
                .store
                .catalog()
                .table_by_role(BitableRole::QuoteDigest)
                .await?;

            let facts = extract_day(&self.pool, date).await?;
            let facts = self.store.facts().upsert_day(facts).await?;

            let client = BitableClient::connect().map_err(BiError::bitable)?;
            // Heal columns deleted in Feishu UI (e.g. 「类型」) before push.
            if let Err(e) = bitable::ensure_otc_fields(&client).await {
                tracing::warn!(error = %e, "ensure_otc_fields failed");
            }
            let push = bitable::push_ledger(&client, &ledger.table_id, date, &facts)
                .await
                .map_err(BiError::bitable)?;

            for f in &facts {
                if let Some(rid) = push.record_ids.get(&f.sync_key) {
                    self.store
                        .facts()
                        .mark_pushed(&f.sync_key, &ledger.table_id, rid)
                        .await?;
                }
            }

            let digest_updated = if let Some(d) = &digest {
                bitable::push_digest(&client, &d.table_id, date, &facts)
                    .await
                    .map_err(BiError::bitable)?
            } else {
                0
            };

            // Site「Best Deposit Quote」→ OTC每日最高报价（按期限×币种最高 Bid）
            let best_rows = best_deposit_facts(date, &facts);
            let (best_updated, best_table_id) = if let Some(t) = &daily_best {
                let best_push = bitable::push_ledger(&client, &t.table_id, date, &best_rows)
                    .await
                    .map_err(BiError::bitable)?;
                (
                    best_push.created + best_push.updated,
                    Some(t.table_id.clone()),
                )
            } else {
                (0, None)
            };

            // Refresh filtered views (incl. 最新日* ExactDate) for dashboards.
            if let Err(e) = bitable::ensure_ledger_chart_views_for_date(
                &client,
                &ledger.table_id,
                Some(date),
            )
            .await
            {
                tracing::warn!(error = %e, "ledger views refresh failed");
            }

            let report = SyncDayReport {
                job_id: job.clone(),
                date: date.to_string(),
                facts: facts.len(),
                created: push.created,
                updated: push.updated,
                deleted: push.deleted,
                digest_updated,
                best_updated,
                ledger_table_id: ledger.table_id,
                digest_table_id: digest.map(|d| d.table_id),
                best_table_id,
            };
            info!(?report, "bi sync_day done");
            Ok(report)
        }
        .await;

        match &result {
            Ok(r) => {
                let _ = self
                    .store
                    .jobs()
                    .finish_ok(
                        &job,
                        r.created as i32,
                        r.updated as i32,
                        r.deleted as i32,
                        r.facts as i32,
                        serde_json::to_value(r).unwrap_or(json!({})),
                    )
                    .await;
            }
            Err(e) => {
                let _ = self.store.jobs().finish_err(&job, &e.to_string()).await;
            }
        }
        result
    }

    pub async fn verify(&self) -> Result<VerifyReport, BiError> {
        let catalog = self.refresh_catalog().await?;
        let sync = self.sync_day(None).await?;
        let client = BitableClient::connect().map_err(BiError::bitable)?;
        let read_back = client
            .search_by_sync_contains(&sync.ledger_table_id, &format!("|{}|", sync.date))
            .await
            .map_err(BiError::bitable)?
            .len();
        let ok = sync.facts > 0 && read_back == sync.facts;
        Ok(VerifyReport {
            ok,
            catalog_tables: catalog.len(),
            sync,
            read_back,
        })
    }

    pub async fn list_catalog(&self) -> Result<Vec<TableCatalog>, BiError> {
        self.store.catalog().list_tables().await
    }

    pub async fn list_dashboards(&self) -> Result<Vec<super::models::DashboardCatalog>, BiError> {
        self.store.catalog().list_dashboards().await
    }

    /// Ensure OTC dashboard exists (copy template if needed) and persist catalog.
    pub async fn ensure_dashboard(&self) -> Result<bitable::DashboardEnsureReport, BiError> {
        let client = BitableClient::connect().map_err(BiError::bitable)?;
        // Prepare ledger views used by charts.
        if let Some(ledger) = self
            .store
            .catalog()
            .table_by_role(BitableRole::QuoteLedger)
            .await?
        {
            let _ = bitable::ensure_ledger_chart_views_for_date(
                &client,
                &ledger.table_id,
                None,
            )
            .await
            .map_err(BiError::bitable)?;
        } else {
            self.refresh_catalog().await?;
            if let Some(ledger) = self
                .store
                .catalog()
                .table_by_role(BitableRole::QuoteLedger)
                .await?
            {
                let _ = bitable::ensure_ledger_chart_views_for_date(
                    &client,
                    &ledger.table_id,
                    None,
                )
                .await
                .map_err(BiError::bitable)?;
            }
        }
        let report = bitable::ensure_otc_dashboard(&client)
            .await
            .map_err(BiError::bitable)?;
        let dashboards = bitable::list_dashboards(&client)
            .await
            .map_err(BiError::bitable)?;
        self.store
            .catalog()
            .replace_dashboards(&client.cfg.app_token, &dashboards)
            .await?;
        Ok(report)
    }

    pub async fn list_facts(
        &self,
        date: Option<NaiveDate>,
    ) -> Result<Vec<super::models::QuoteFact>, BiError> {
        let date = resolve_quote_date(&self.pool, date).await?;
        self.store.facts().list_by_date(date).await
    }

    pub async fn recent_jobs(&self, limit: i64) -> Result<Vec<super::models::SyncJob>, BiError> {
        self.store.jobs().recent(limit).await
    }
}

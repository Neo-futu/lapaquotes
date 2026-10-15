use chrono::NaiveDate;
use cornucopia::deadpool_postgres::Pool;
use serde_json::Value;

use crate::bi::error::BiError;
use crate::bi::models::SyncJob;

use super::new_id;

pub struct JobStore<'a> {
    pub(super) pool: &'a Pool,
}

impl JobStore<'_> {
    pub async fn start(
        &self,
        job_type: &str,
        quote_date: Option<NaiveDate>,
    ) -> Result<String, BiError> {
        let id = new_id();
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| BiError::Pool(e.to_string()))?;
        client
            .execute(
                r#"
                INSERT INTO "BiSyncJob" (id, job_type, quote_date, status, detail)
                VALUES ($1, $2, $3, 'running', '{}'::jsonb)
                "#,
                &[&id, &job_type, &quote_date],
            )
            .await?;
        Ok(id)
    }

    pub async fn finish_ok(
        &self,
        id: &str,
        created: i32,
        updated: i32,
        deleted: i32,
        facts: i32,
        detail: Value,
    ) -> Result<(), BiError> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| BiError::Pool(e.to_string()))?;
        client
            .execute(
                r#"
                UPDATE "BiSyncJob" SET
                    status = 'ok',
                    created_n = $2,
                    updated_n = $3,
                    deleted_n = $4,
                    facts_n = $5,
                    detail = $6::jsonb,
                    finished_at = CURRENT_TIMESTAMP
                WHERE id = $1
                "#,
                &[
                    &id,
                    &created,
                    &updated,
                    &deleted,
                    &facts,
                    &serde_json::to_string(&detail).unwrap_or_else(|_| "{}".into()),
                ],
            )
            .await?;
        Ok(())
    }

    pub async fn finish_err(&self, id: &str, error: &str) -> Result<(), BiError> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| BiError::Pool(e.to_string()))?;
        client
            .execute(
                r#"
                UPDATE "BiSyncJob" SET
                    status = 'error',
                    error = $2,
                    finished_at = CURRENT_TIMESTAMP
                WHERE id = $1
                "#,
                &[&id, &error],
            )
            .await?;
        Ok(())
    }

    pub async fn recent(&self, limit: i64) -> Result<Vec<SyncJob>, BiError> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| BiError::Pool(e.to_string()))?;
        let rows = client
            .query(
                r#"
                SELECT id, job_type, quote_date, status,
                       created_n, updated_n, deleted_n, facts_n, error, detail::text
                FROM "BiSyncJob"
                ORDER BY created_at DESC
                LIMIT $1
                "#,
                &[&limit],
            )
            .await?;
        Ok(rows
            .into_iter()
            .map(|r| {
                let detail_txt: String = r.get(9);
                let detail = serde_json::from_str(&detail_txt).unwrap_or(Value::Object(Default::default()));
                SyncJob {
                    id: r.get(0),
                    job_type: r.get(1),
                    quote_date: r.get(2),
                    status: r.get(3),
                    created_n: r.get(4),
                    updated_n: r.get(5),
                    deleted_n: r.get(6),
                    facts_n: r.get(7),
                    error: r.get(8),
                    detail,
                }
            })
            .collect())
    }
}

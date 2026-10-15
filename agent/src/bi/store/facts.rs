use chrono::NaiveDate;
use cornucopia::deadpool_postgres::Pool;
use rust_decimal::Decimal;

use crate::bi::error::BiError;
use crate::bi::models::QuoteFact;

use super::new_id;

pub struct FactStore<'a> {
    pub(super) pool: &'a Pool,
}

impl FactStore<'_> {
    /// Upsert facts for a day; returns (inserted_or_updated_count, facts with ids).
    pub async fn upsert_day(&self, facts: Vec<QuoteFact>) -> Result<Vec<QuoteFact>, BiError> {
        if facts.is_empty() {
            return Ok(vec![]);
        }
        let mut client = self
            .pool
            .get()
            .await
            .map_err(|e| BiError::Pool(e.to_string()))?;
        let tx = client.transaction().await?;
        let mut out = Vec::with_capacity(facts.len());

        for mut f in facts {
            let bid = f.bid.map(|v| Decimal::try_from(v).unwrap_or(Decimal::ZERO));
            let ask = f.ask.map(|v| Decimal::try_from(v).unwrap_or(Decimal::ZERO));
            let row = tx
                .query_opt(
                    r#"SELECT id, bitable_record_id, bitable_table_id FROM "BiQuoteFact" WHERE sync_key = $1"#,
                    &[&f.sync_key],
                )
                .await?;

            if let Some(existing) = row {
                let id: String = existing.get(0);
                let record_id: Option<String> = existing.get(1);
                let table_id: Option<String> = existing.get(2);
                tx.execute(
                    r#"
                    UPDATE "BiQuoteFact" SET
                        quote_date = $2,
                        kind = $3,
                        institution_name = $4,
                        term = $5,
                        currency = $6,
                        bond_code = $7,
                        bond_name = $8,
                        bid = $9,
                        ask = $10,
                        note = $11,
                        updated_at = CURRENT_TIMESTAMP
                    WHERE id = $1
                    "#,
                    &[
                        &id,
                        &f.quote_date,
                        &f.kind,
                        &f.institution_name,
                        &f.term,
                        &f.currency,
                        &f.bond_code,
                        &f.bond_name,
                        &bid,
                        &ask,
                        &f.note,
                    ],
                )
                .await?;
                f.id = id;
                f.bitable_record_id = record_id;
                f.bitable_table_id = table_id;
            } else {
                if f.id.is_empty() {
                    f.id = new_id();
                }
                tx.execute(
                    r#"
                    INSERT INTO "BiQuoteFact" (
                        id, sync_key, quote_date, kind, institution_name,
                        term, currency, bond_code, bond_name, bid, ask, note
                    ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)
                    "#,
                    &[
                        &f.id,
                        &f.sync_key,
                        &f.quote_date,
                        &f.kind,
                        &f.institution_name,
                        &f.term,
                        &f.currency,
                        &f.bond_code,
                        &f.bond_name,
                        &bid,
                        &ask,
                        &f.note,
                    ],
                )
                .await?;
            }
            out.push(f);
        }

        tx.commit().await?;
        Ok(out)
    }

    pub async fn list_by_date(&self, date: NaiveDate) -> Result<Vec<QuoteFact>, BiError> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| BiError::Pool(e.to_string()))?;
        let rows = client
            .query(
                r#"
                SELECT id, sync_key, quote_date, kind, institution_name,
                       term, currency, bond_code, bond_name, bid, ask, note,
                       bitable_table_id, bitable_record_id, pushed_at
                FROM "BiQuoteFact"
                WHERE quote_date = $1
                ORDER BY kind, institution_name, term NULLS LAST, bond_code NULLS LAST
                "#,
                &[&date],
            )
            .await?;
        Ok(rows.into_iter().map(row_to_fact).collect())
    }

    pub async fn mark_pushed(
        &self,
        sync_key: &str,
        table_id: &str,
        record_id: &str,
    ) -> Result<(), BiError> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| BiError::Pool(e.to_string()))?;
        client
            .execute(
                r#"
                UPDATE "BiQuoteFact" SET
                    bitable_table_id = $2,
                    bitable_record_id = $3,
                    pushed_at = CURRENT_TIMESTAMP,
                    updated_at = CURRENT_TIMESTAMP
                WHERE sync_key = $1
                "#,
                &[&sync_key, &table_id, &record_id],
            )
            .await?;
        Ok(())
    }

}

fn row_to_fact(row: cornucopia::tokio_postgres::Row) -> QuoteFact {
    let bid: Option<Decimal> = row.get(9);
    let ask: Option<Decimal> = row.get(10);
    QuoteFact {
        id: row.get(0),
        sync_key: row.get(1),
        quote_date: row.get(2),
        kind: row.get(3),
        institution_name: row.get(4),
        term: row.get(5),
        currency: row.get(6),
        bond_code: row.get(7),
        bond_name: row.get(8),
        bid: bid.and_then(|d| d.try_into().ok()),
        ask: ask.and_then(|d| d.try_into().ok()),
        note: row.get(11),
        bitable_table_id: row.get(12),
        bitable_record_id: row.get(13),
        pushed_at: row.get(14),
    }
}

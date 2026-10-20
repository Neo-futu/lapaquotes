use cornucopia::deadpool_postgres::Pool;

use crate::bi::error::BiError;
use crate::bi::models::{
    BitableRole, DashboardCatalog, FieldCatalog, TableCatalog, ViewCatalog,
};
use crate::bi::bitable::DashboardInfo;

pub struct CatalogStore<'a> {
    pub(super) pool: &'a Pool,
}

impl CatalogStore<'_> {
    pub async fn replace_catalog(
        &self,
        app_token: &str,
        tables: &[TableCatalog],
    ) -> Result<(), BiError> {
        let mut client = self
            .pool
            .get()
            .await
            .map_err(|e| BiError::Pool(e.to_string()))?;
        let tx = client.transaction().await?;

        // Remove stale catalog rows for this app (cascade views/fields).
        tx.execute(
            r#"DELETE FROM "BiBitableTable" WHERE app_token = $1"#,
            &[&app_token],
        )
        .await?;

        for t in tables {
            tx.execute(
                r#"
                INSERT INTO "BiBitableTable"
                    (id, app_token, name, role, default_view_id, revision, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, CURRENT_TIMESTAMP)
                "#,
                &[
                    &t.table_id,
                    &app_token,
                    &t.name,
                    &t.role.as_str(),
                    &t.default_view_id,
                    &t.revision,
                ],
            )
            .await?;

            for v in &t.views {
                tx.execute(
                    r#"
                    INSERT INTO "BiBitableView" (id, table_id, name, view_type, updated_at)
                    VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP)
                    "#,
                    &[&v.view_id, &t.table_id, &v.name, &v.view_type],
                )
                .await?;
            }
            for f in &t.fields {
                tx.execute(
                    r#"
                    INSERT INTO "BiBitableField"
                        (id, table_id, name, field_type, ui_type, is_primary, updated_at)
                    VALUES ($1, $2, $3, $4, $5, $6, CURRENT_TIMESTAMP)
                    "#,
                    &[
                        &f.field_id,
                        &t.table_id,
                        &f.name,
                        &f.field_type,
                        &f.ui_type,
                        &f.is_primary,
                    ],
                )
                .await?;
            }
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn list_tables(&self) -> Result<Vec<TableCatalog>, BiError> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| BiError::Pool(e.to_string()))?;
        let rows = client
            .query(
                r#"
                SELECT id, name, role, default_view_id, revision
                FROM "BiBitableTable"
                ORDER BY
                  CASE role
                    WHEN 'quote_ledger' THEN 0
                    WHEN 'quote_digest' THEN 1
                    WHEN 'daily_best' THEN 2
                    ELSE 3
                  END,
                  name
                "#,
                &[],
            )
            .await?;

        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
            let table_id: String = row.get(0);
            let name: String = row.get(1);
            let role = BitableRole::parse(row.get::<_, &str>(2));
            let default_view_id: Option<String> = row.get(3);
            let revision: Option<i32> = row.get(4);

            let views = client
                .query(
                    r#"SELECT id, name, view_type FROM "BiBitableView" WHERE table_id = $1 ORDER BY name"#,
                    &[&table_id],
                )
                .await?
                .into_iter()
                .map(|r| ViewCatalog {
                    view_id: r.get(0),
                    name: r.get(1),
                    view_type: r.get(2),
                })
                .collect();

            let fields = client
                .query(
                    r#"
                    SELECT id, name, field_type, ui_type, is_primary
                    FROM "BiBitableField"
                    WHERE table_id = $1
                    ORDER BY is_primary DESC, name
                    "#,
                    &[&table_id],
                )
                .await?
                .into_iter()
                .map(|r| FieldCatalog {
                    field_id: r.get(0),
                    name: r.get(1),
                    field_type: r.get(2),
                    ui_type: r.get(3),
                    is_primary: r.get(4),
                })
                .collect();

            out.push(TableCatalog {
                table_id,
                name,
                role,
                default_view_id,
                revision,
                views,
                fields,
            });
        }
        Ok(out)
    }

    pub async fn table_by_role(&self, role: BitableRole) -> Result<Option<TableCatalog>, BiError> {
        Ok(self
            .list_tables()
            .await?
            .into_iter()
            .find(|t| t.role == role))
    }

    pub async fn replace_dashboards(
        &self,
        app_token: &str,
        dashboards: &[DashboardInfo],
    ) -> Result<(), BiError> {
        let mut client = self
            .pool
            .get()
            .await
            .map_err(|e| BiError::Pool(e.to_string()))?;
        let tx = client.transaction().await?;
        tx.execute(
            r#"DELETE FROM "BiBitableDashboard" WHERE app_token = $1"#,
            &[&app_token],
        )
        .await?;
        for d in dashboards {
            tx.execute(
                r#"
                INSERT INTO "BiBitableDashboard"
                    (id, app_token, name, role, source_block_id, url, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, CURRENT_TIMESTAMP)
                "#,
                &[
                    &d.block_id,
                    &app_token,
                    &d.name,
                    &d.role,
                    &d.source_block_id,
                    &d.url,
                ],
            )
            .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn list_dashboards(&self) -> Result<Vec<DashboardCatalog>, BiError> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| BiError::Pool(e.to_string()))?;
        let rows = client
            .query(
                r#"
                SELECT id, name, role, url, source_block_id
                FROM "BiBitableDashboard"
                ORDER BY
                  CASE role
                    WHEN 'quote_dashboard' THEN 0
                    WHEN 'legacy_commodity' THEN 1
                    ELSE 2
                  END,
                  name
                "#,
                &[],
            )
            .await?;
        Ok(rows
            .into_iter()
            .map(|r| DashboardCatalog {
                block_id: r.get(0),
                name: r.get(1),
                role: r.get(2),
                url: r.get(3),
                source_block_id: r.get(4),
            })
            .collect())
    }
}

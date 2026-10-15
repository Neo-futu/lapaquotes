//! Postgres persistence for BI tables.

mod catalog;
mod facts;
mod jobs;

pub use catalog::CatalogStore;
pub use facts::FactStore;
pub use jobs::JobStore;

use cornucopia::deadpool_postgres::Pool;

use super::error::BiError;

pub struct BiStore {
    pool: Pool,
}

impl BiStore {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    pub fn catalog(&self) -> CatalogStore<'_> {
        CatalogStore { pool: &self.pool }
    }

    pub fn facts(&self) -> FactStore<'_> {
        FactStore { pool: &self.pool }
    }

    pub fn jobs(&self) -> JobStore<'_> {
        JobStore { pool: &self.pool }
    }

    pub async fn ping(&self) -> Result<(), BiError> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| BiError::Pool(e.to_string()))?;
        client.simple_query("SELECT 1").await?;
        Ok(())
    }
}

pub fn new_id() -> String {
    cuid2::create_id()
}

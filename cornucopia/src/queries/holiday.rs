// This file was generated with `cornucopia`. Do not modify.

#[derive(Debug, Clone, PartialEq)]
pub struct HolidaysByRegion {
    pub holiday_date: chrono::NaiveDate,
    pub name: String,
}
pub struct HolidaysByRegionBorrowed<'a> {
    pub holiday_date: chrono::NaiveDate,
    pub name: &'a str,
}
impl<'a> From<HolidaysByRegionBorrowed<'a>> for HolidaysByRegion {
    fn from(HolidaysByRegionBorrowed { holiday_date, name }: HolidaysByRegionBorrowed<'a>) -> Self {
        Self {
            holiday_date,
            name: name.into(),
        }
    }
}
use crate::client::async_::GenericClient;
use futures::{self, StreamExt, TryStreamExt};
pub struct HolidaysByRegionQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<HolidaysByRegionBorrowed, tokio_postgres::Error>,
    mapper: fn(HolidaysByRegionBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> HolidaysByRegionQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(HolidaysByRegionBorrowed) -> R,
    ) -> HolidaysByRegionQuery<'c, 'a, 's, C, R, N> {
        HolidaysByRegionQuery {
            client: self.client,
            params: self.params,
            query: self.query,
            cached: self.cached,
            extractor: self.extractor,
            mapper,
        }
    }
    pub async fn one(self) -> Result<T, tokio_postgres::Error> {
        let row =
            crate::client::async_::one(self.client, self.query, &self.params, self.cached).await?;
        Ok((self.mapper)((self.extractor)(&row)?))
    }
    pub async fn all(self) -> Result<Vec<T>, tokio_postgres::Error> {
        self.iter().await?.try_collect().await
    }
    pub async fn opt(self) -> Result<Option<T>, tokio_postgres::Error> {
        let opt_row =
            crate::client::async_::opt(self.client, self.query, &self.params, self.cached).await?;
        Ok(opt_row
            .map(|row| {
                let extracted = (self.extractor)(&row)?;
                Ok((self.mapper)(extracted))
            })
            .transpose()?)
    }
    pub async fn iter(
        self,
    ) -> Result<
        impl futures::Stream<Item = Result<T, tokio_postgres::Error>> + 'c,
        tokio_postgres::Error,
    > {
        let stream = crate::client::async_::raw(
            self.client,
            self.query,
            crate::slice_iter(&self.params),
            self.cached,
        )
        .await?;
        let mapped = stream
            .map(move |res| {
                res.and_then(|row| {
                    let extracted = (self.extractor)(&row)?;
                    Ok((self.mapper)(extracted))
                })
            })
            .into_stream();
        Ok(mapped)
    }
}
pub struct HolidaysByRegionStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn holidays_by_region() -> HolidaysByRegionStmt {
    HolidaysByRegionStmt(
        "SELECT \"Holiday\".holiday_date, \"Holiday\".name FROM \"Holiday\" WHERE (CAST($1 AS TEXT) IS NULL OR \"Holiday\".region = CAST($1 AS TEXT)) ORDER BY \"Holiday\".holiday_date",
        None,
    )
}
impl HolidaysByRegionStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub fn bind<'c, 'a, 's, C: GenericClient, T1: crate::StringSql>(
        &'s self,
        client: &'c C,
        region: &'a Option<T1>,
    ) -> HolidaysByRegionQuery<'c, 'a, 's, C, HolidaysByRegion, 1> {
        HolidaysByRegionQuery {
            client,
            params: [region],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<HolidaysByRegionBorrowed, tokio_postgres::Error> {
                Ok(HolidaysByRegionBorrowed {
                    holiday_date: row.try_get(0)?,
                    name: row.try_get(1)?,
                })
            },
            mapper: |it| HolidaysByRegion::from(it),
        }
    }
}

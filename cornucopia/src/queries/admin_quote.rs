// This file was generated with `cornucopia`. Do not modify.

#[derive(Debug)]
pub struct DepositListParams<T1: crate::StringSql> {
    pub from: Option<chrono::NaiveDate>,
    pub to: Option<chrono::NaiveDate>,
    pub institution_id: Option<T1>,
}
#[derive(Debug)]
pub struct DepositCreateParams<
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::StringSql,
    T4: crate::StringSql,
    T5: crate::StringSql,
> {
    pub id: T1,
    pub quote_date: chrono::NaiveDate,
    pub institution_id: T2,
    pub term: T3,
    pub term_sort: i32,
    pub currency: T4,
    pub bid_rate: Option<rust_decimal::Decimal>,
    pub ask_rate: Option<rust_decimal::Decimal>,
    pub note: Option<T5>,
}
#[derive(Debug)]
pub struct DepositUpdateParams<
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::StringSql,
    T4: crate::StringSql,
    T5: crate::StringSql,
> {
    pub quote_date: chrono::NaiveDate,
    pub institution_id: T1,
    pub term: T2,
    pub term_sort: i32,
    pub currency: T3,
    pub bid_rate: Option<rust_decimal::Decimal>,
    pub ask_rate: Option<rust_decimal::Decimal>,
    pub note: Option<T4>,
    pub id: T5,
}
#[derive(Debug)]
pub struct BondListParams<T1: crate::StringSql> {
    pub from: Option<chrono::NaiveDate>,
    pub to: Option<chrono::NaiveDate>,
    pub institution_id: Option<T1>,
}
#[derive(Debug)]
pub struct BondCreateParams<
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::StringSql,
    T4: crate::StringSql,
    T5: crate::StringSql,
    T6: crate::StringSql,
    T7: crate::StringSql,
> {
    pub id: T1,
    pub quote_date: chrono::NaiveDate,
    pub institution_id: T2,
    pub bond_code: T3,
    pub bond_name: Option<T4>,
    pub currency: T5,
    pub maturity_date: Option<chrono::NaiveDate>,
    pub coupon_rate: Option<rust_decimal::Decimal>,
    pub bid_yield: Option<rust_decimal::Decimal>,
    pub ask_yield: Option<rust_decimal::Decimal>,
    pub bid_price: Option<rust_decimal::Decimal>,
    pub ask_price: Option<rust_decimal::Decimal>,
    pub direction: Option<T6>,
    pub note: Option<T7>,
}
#[derive(Debug)]
pub struct BondUpdateParams<
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::StringSql,
    T4: crate::StringSql,
    T5: crate::StringSql,
    T6: crate::StringSql,
    T7: crate::StringSql,
> {
    pub quote_date: chrono::NaiveDate,
    pub institution_id: T1,
    pub bond_code: T2,
    pub bond_name: Option<T3>,
    pub currency: T4,
    pub maturity_date: Option<chrono::NaiveDate>,
    pub coupon_rate: Option<rust_decimal::Decimal>,
    pub bid_yield: Option<rust_decimal::Decimal>,
    pub ask_yield: Option<rust_decimal::Decimal>,
    pub bid_price: Option<rust_decimal::Decimal>,
    pub ask_price: Option<rust_decimal::Decimal>,
    pub direction: Option<T5>,
    pub note: Option<T6>,
    pub id: T7,
}
#[derive(Debug, Clone, PartialEq)]
pub struct DepositList {
    pub id: String,
    pub created_at: chrono::NaiveDateTime,
    pub quote_date: chrono::NaiveDate,
    pub institution_id: String,
    pub institution_name: String,
    pub institution_short_name: Option<String>,
    pub term: String,
    pub term_sort: i32,
    pub currency: String,
    pub bid_rate: Option<rust_decimal::Decimal>,
    pub ask_rate: Option<rust_decimal::Decimal>,
    pub note: Option<String>,
}
pub struct DepositListBorrowed<'a> {
    pub id: &'a str,
    pub created_at: chrono::NaiveDateTime,
    pub quote_date: chrono::NaiveDate,
    pub institution_id: &'a str,
    pub institution_name: &'a str,
    pub institution_short_name: Option<&'a str>,
    pub term: &'a str,
    pub term_sort: i32,
    pub currency: &'a str,
    pub bid_rate: Option<rust_decimal::Decimal>,
    pub ask_rate: Option<rust_decimal::Decimal>,
    pub note: Option<&'a str>,
}
impl<'a> From<DepositListBorrowed<'a>> for DepositList {
    fn from(
        DepositListBorrowed {
            id,
            created_at,
            quote_date,
            institution_id,
            institution_name,
            institution_short_name,
            term,
            term_sort,
            currency,
            bid_rate,
            ask_rate,
            note,
        }: DepositListBorrowed<'a>,
    ) -> Self {
        Self {
            id: id.into(),
            created_at,
            quote_date,
            institution_id: institution_id.into(),
            institution_name: institution_name.into(),
            institution_short_name: institution_short_name.map(|v| v.into()),
            term: term.into(),
            term_sort,
            currency: currency.into(),
            bid_rate,
            ask_rate,
            note: note.map(|v| v.into()),
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct DepositPage {
    pub id: String,
    pub created_at: chrono::NaiveDateTime,
    pub quote_date: chrono::NaiveDate,
    pub institution_id: String,
    pub term: String,
    pub term_sort: i32,
    pub currency: String,
    pub bid_rate: Option<rust_decimal::Decimal>,
    pub ask_rate: Option<rust_decimal::Decimal>,
    pub note: Option<String>,
}
pub struct DepositPageBorrowed<'a> {
    pub id: &'a str,
    pub created_at: chrono::NaiveDateTime,
    pub quote_date: chrono::NaiveDate,
    pub institution_id: &'a str,
    pub term: &'a str,
    pub term_sort: i32,
    pub currency: &'a str,
    pub bid_rate: Option<rust_decimal::Decimal>,
    pub ask_rate: Option<rust_decimal::Decimal>,
    pub note: Option<&'a str>,
}
impl<'a> From<DepositPageBorrowed<'a>> for DepositPage {
    fn from(
        DepositPageBorrowed {
            id,
            created_at,
            quote_date,
            institution_id,
            term,
            term_sort,
            currency,
            bid_rate,
            ask_rate,
            note,
        }: DepositPageBorrowed<'a>,
    ) -> Self {
        Self {
            id: id.into(),
            created_at,
            quote_date,
            institution_id: institution_id.into(),
            term: term.into(),
            term_sort,
            currency: currency.into(),
            bid_rate,
            ask_rate,
            note: note.map(|v| v.into()),
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct BondList {
    pub id: String,
    pub created_at: chrono::NaiveDateTime,
    pub quote_date: chrono::NaiveDate,
    pub institution_id: String,
    pub institution_name: String,
    pub institution_short_name: Option<String>,
    pub bond_code: String,
    pub bond_name: Option<String>,
    pub currency: String,
    pub maturity_date: Option<chrono::NaiveDate>,
    pub coupon_rate: Option<rust_decimal::Decimal>,
    pub bid_yield: Option<rust_decimal::Decimal>,
    pub ask_yield: Option<rust_decimal::Decimal>,
    pub bid_price: Option<rust_decimal::Decimal>,
    pub ask_price: Option<rust_decimal::Decimal>,
    pub direction: Option<String>,
    pub note: Option<String>,
}
pub struct BondListBorrowed<'a> {
    pub id: &'a str,
    pub created_at: chrono::NaiveDateTime,
    pub quote_date: chrono::NaiveDate,
    pub institution_id: &'a str,
    pub institution_name: &'a str,
    pub institution_short_name: Option<&'a str>,
    pub bond_code: &'a str,
    pub bond_name: Option<&'a str>,
    pub currency: &'a str,
    pub maturity_date: Option<chrono::NaiveDate>,
    pub coupon_rate: Option<rust_decimal::Decimal>,
    pub bid_yield: Option<rust_decimal::Decimal>,
    pub ask_yield: Option<rust_decimal::Decimal>,
    pub bid_price: Option<rust_decimal::Decimal>,
    pub ask_price: Option<rust_decimal::Decimal>,
    pub direction: Option<&'a str>,
    pub note: Option<&'a str>,
}
impl<'a> From<BondListBorrowed<'a>> for BondList {
    fn from(
        BondListBorrowed {
            id,
            created_at,
            quote_date,
            institution_id,
            institution_name,
            institution_short_name,
            bond_code,
            bond_name,
            currency,
            maturity_date,
            coupon_rate,
            bid_yield,
            ask_yield,
            bid_price,
            ask_price,
            direction,
            note,
        }: BondListBorrowed<'a>,
    ) -> Self {
        Self {
            id: id.into(),
            created_at,
            quote_date,
            institution_id: institution_id.into(),
            institution_name: institution_name.into(),
            institution_short_name: institution_short_name.map(|v| v.into()),
            bond_code: bond_code.into(),
            bond_name: bond_name.map(|v| v.into()),
            currency: currency.into(),
            maturity_date,
            coupon_rate,
            bid_yield,
            ask_yield,
            bid_price,
            ask_price,
            direction: direction.map(|v| v.into()),
            note: note.map(|v| v.into()),
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct BondPage {
    pub id: String,
    pub created_at: chrono::NaiveDateTime,
    pub quote_date: chrono::NaiveDate,
    pub institution_id: String,
    pub bond_code: String,
    pub bond_name: Option<String>,
    pub currency: String,
    pub maturity_date: Option<chrono::NaiveDate>,
    pub coupon_rate: Option<rust_decimal::Decimal>,
    pub bid_yield: Option<rust_decimal::Decimal>,
    pub ask_yield: Option<rust_decimal::Decimal>,
    pub bid_price: Option<rust_decimal::Decimal>,
    pub ask_price: Option<rust_decimal::Decimal>,
    pub direction: Option<String>,
    pub note: Option<String>,
}
pub struct BondPageBorrowed<'a> {
    pub id: &'a str,
    pub created_at: chrono::NaiveDateTime,
    pub quote_date: chrono::NaiveDate,
    pub institution_id: &'a str,
    pub bond_code: &'a str,
    pub bond_name: Option<&'a str>,
    pub currency: &'a str,
    pub maturity_date: Option<chrono::NaiveDate>,
    pub coupon_rate: Option<rust_decimal::Decimal>,
    pub bid_yield: Option<rust_decimal::Decimal>,
    pub ask_yield: Option<rust_decimal::Decimal>,
    pub bid_price: Option<rust_decimal::Decimal>,
    pub ask_price: Option<rust_decimal::Decimal>,
    pub direction: Option<&'a str>,
    pub note: Option<&'a str>,
}
impl<'a> From<BondPageBorrowed<'a>> for BondPage {
    fn from(
        BondPageBorrowed {
            id,
            created_at,
            quote_date,
            institution_id,
            bond_code,
            bond_name,
            currency,
            maturity_date,
            coupon_rate,
            bid_yield,
            ask_yield,
            bid_price,
            ask_price,
            direction,
            note,
        }: BondPageBorrowed<'a>,
    ) -> Self {
        Self {
            id: id.into(),
            created_at,
            quote_date,
            institution_id: institution_id.into(),
            bond_code: bond_code.into(),
            bond_name: bond_name.map(|v| v.into()),
            currency: currency.into(),
            maturity_date,
            coupon_rate,
            bid_yield,
            ask_yield,
            bid_price,
            ask_price,
            direction: direction.map(|v| v.into()),
            note: note.map(|v| v.into()),
        }
    }
}
use crate::client::async_::GenericClient;
use futures::{self, StreamExt, TryStreamExt};
pub struct DepositListQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<DepositListBorrowed, tokio_postgres::Error>,
    mapper: fn(DepositListBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> DepositListQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(DepositListBorrowed) -> R,
    ) -> DepositListQuery<'c, 'a, 's, C, R, N> {
        DepositListQuery {
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
pub struct DepositPageQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<DepositPageBorrowed, tokio_postgres::Error>,
    mapper: fn(DepositPageBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> DepositPageQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(DepositPageBorrowed) -> R,
    ) -> DepositPageQuery<'c, 'a, 's, C, R, N> {
        DepositPageQuery {
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
pub struct ChronoNaiveDateTimeQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<chrono::NaiveDateTime, tokio_postgres::Error>,
    mapper: fn(chrono::NaiveDateTime) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> ChronoNaiveDateTimeQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(chrono::NaiveDateTime) -> R,
    ) -> ChronoNaiveDateTimeQuery<'c, 'a, 's, C, R, N> {
        ChronoNaiveDateTimeQuery {
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
pub struct StringQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<&str, tokio_postgres::Error>,
    mapper: fn(&str) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> StringQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(self, mapper: fn(&str) -> R) -> StringQuery<'c, 'a, 's, C, R, N> {
        StringQuery {
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
pub struct BondListQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<BondListBorrowed, tokio_postgres::Error>,
    mapper: fn(BondListBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> BondListQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(self, mapper: fn(BondListBorrowed) -> R) -> BondListQuery<'c, 'a, 's, C, R, N> {
        BondListQuery {
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
pub struct BondPageQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<BondPageBorrowed, tokio_postgres::Error>,
    mapper: fn(BondPageBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> BondPageQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(self, mapper: fn(BondPageBorrowed) -> R) -> BondPageQuery<'c, 'a, 's, C, R, N> {
        BondPageQuery {
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
pub struct DepositListStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn deposit_list() -> DepositListStmt {
    DepositListStmt(
        "SELECT \"DepositQuote\".id, \"DepositQuote\".created_at, \"DepositQuote\".quote_date, \"DepositQuote\".institution_id, \"Institution\".name AS institution_name, \"Institution\".short_name AS institution_short_name, \"DepositQuote\".term, \"DepositQuote\".term_sort, \"DepositQuote\".currency, \"DepositQuote\".bid_rate, \"DepositQuote\".ask_rate, \"DepositQuote\".note FROM \"DepositQuote\" INNER JOIN \"Institution\" ON \"Institution\".id = \"DepositQuote\".institution_id WHERE (CAST($1 AS DATE) IS NULL OR \"DepositQuote\".quote_date >= $1) AND (CAST($2 AS DATE) IS NULL OR \"DepositQuote\".quote_date <= $2) AND (CAST($3 AS TEXT) IS NULL OR \"DepositQuote\".institution_id = $3) ORDER BY \"DepositQuote\".quote_date DESC, \"Institution\".name, \"DepositQuote\".currency, \"DepositQuote\".term_sort",
        None,
    )
}
impl DepositListStmt {
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
        from: &'a Option<chrono::NaiveDate>,
        to: &'a Option<chrono::NaiveDate>,
        institution_id: &'a Option<T1>,
    ) -> DepositListQuery<'c, 'a, 's, C, DepositList, 3> {
        DepositListQuery {
            client,
            params: [from, to, institution_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor:
                |row: &tokio_postgres::Row| -> Result<DepositListBorrowed, tokio_postgres::Error> {
                    Ok(DepositListBorrowed {
                        id: row.try_get(0)?,
                        created_at: row.try_get(1)?,
                        quote_date: row.try_get(2)?,
                        institution_id: row.try_get(3)?,
                        institution_name: row.try_get(4)?,
                        institution_short_name: row.try_get(5)?,
                        term: row.try_get(6)?,
                        term_sort: row.try_get(7)?,
                        currency: row.try_get(8)?,
                        bid_rate: row.try_get(9)?,
                        ask_rate: row.try_get(10)?,
                        note: row.try_get(11)?,
                    })
                },
            mapper: |it| DepositList::from(it),
        }
    }
}
impl<'c, 'a, 's, C: GenericClient, T1: crate::StringSql>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        DepositListParams<T1>,
        DepositListQuery<'c, 'a, 's, C, DepositList, 3>,
        C,
    > for DepositListStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a DepositListParams<T1>,
    ) -> DepositListQuery<'c, 'a, 's, C, DepositList, 3> {
        self.bind(client, &params.from, &params.to, &params.institution_id)
    }
}
pub struct DepositPageStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn deposit_page() -> DepositPageStmt {
    DepositPageStmt(
        "SELECT \"DepositQuote\".id, \"DepositQuote\".created_at, \"DepositQuote\".quote_date, \"DepositQuote\".institution_id, \"DepositQuote\".term, \"DepositQuote\".term_sort, \"DepositQuote\".currency, \"DepositQuote\".bid_rate, \"DepositQuote\".ask_rate, \"DepositQuote\".note FROM \"DepositQuote\" WHERE \"DepositQuote\".id = $1",
        None,
    )
}
impl DepositPageStmt {
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
        id: &'a T1,
    ) -> DepositPageQuery<'c, 'a, 's, C, DepositPage, 1> {
        DepositPageQuery {
            client,
            params: [id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor:
                |row: &tokio_postgres::Row| -> Result<DepositPageBorrowed, tokio_postgres::Error> {
                    Ok(DepositPageBorrowed {
                        id: row.try_get(0)?,
                        created_at: row.try_get(1)?,
                        quote_date: row.try_get(2)?,
                        institution_id: row.try_get(3)?,
                        term: row.try_get(4)?,
                        term_sort: row.try_get(5)?,
                        currency: row.try_get(6)?,
                        bid_rate: row.try_get(7)?,
                        ask_rate: row.try_get(8)?,
                        note: row.try_get(9)?,
                    })
                },
            mapper: |it| DepositPage::from(it),
        }
    }
}
pub struct DepositCreateStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn deposit_create() -> DepositCreateStmt {
    DepositCreateStmt(
        "INSERT INTO \"DepositQuote\" (id, quote_date, institution_id, term, term_sort, currency, bid_rate, ask_rate, note) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING created_at",
        None,
    )
}
impl DepositCreateStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub fn bind<
        'c,
        'a,
        's,
        C: GenericClient,
        T1: crate::StringSql,
        T2: crate::StringSql,
        T3: crate::StringSql,
        T4: crate::StringSql,
        T5: crate::StringSql,
    >(
        &'s self,
        client: &'c C,
        id: &'a T1,
        quote_date: &'a chrono::NaiveDate,
        institution_id: &'a T2,
        term: &'a T3,
        term_sort: &'a i32,
        currency: &'a T4,
        bid_rate: &'a Option<rust_decimal::Decimal>,
        ask_rate: &'a Option<rust_decimal::Decimal>,
        note: &'a Option<T5>,
    ) -> ChronoNaiveDateTimeQuery<'c, 'a, 's, C, chrono::NaiveDateTime, 9> {
        ChronoNaiveDateTimeQuery {
            client,
            params: [
                id,
                quote_date,
                institution_id,
                term,
                term_sort,
                currency,
                bid_rate,
                ask_rate,
                note,
            ],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it,
        }
    }
}
impl<
    'c,
    'a,
    's,
    C: GenericClient,
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::StringSql,
    T4: crate::StringSql,
    T5: crate::StringSql,
>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        DepositCreateParams<T1, T2, T3, T4, T5>,
        ChronoNaiveDateTimeQuery<'c, 'a, 's, C, chrono::NaiveDateTime, 9>,
        C,
    > for DepositCreateStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a DepositCreateParams<T1, T2, T3, T4, T5>,
    ) -> ChronoNaiveDateTimeQuery<'c, 'a, 's, C, chrono::NaiveDateTime, 9> {
        self.bind(
            client,
            &params.id,
            &params.quote_date,
            &params.institution_id,
            &params.term,
            &params.term_sort,
            &params.currency,
            &params.bid_rate,
            &params.ask_rate,
            &params.note,
        )
    }
}
pub struct DepositUpdateStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn deposit_update() -> DepositUpdateStmt {
    DepositUpdateStmt(
        "UPDATE \"DepositQuote\" SET quote_date = $1, institution_id = $2, term = $3, term_sort = $4, currency = $5, bid_rate = $6, ask_rate = $7, note = $8 WHERE id = $9 RETURNING id",
        None,
    )
}
impl DepositUpdateStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub fn bind<
        'c,
        'a,
        's,
        C: GenericClient,
        T1: crate::StringSql,
        T2: crate::StringSql,
        T3: crate::StringSql,
        T4: crate::StringSql,
        T5: crate::StringSql,
    >(
        &'s self,
        client: &'c C,
        quote_date: &'a chrono::NaiveDate,
        institution_id: &'a T1,
        term: &'a T2,
        term_sort: &'a i32,
        currency: &'a T3,
        bid_rate: &'a Option<rust_decimal::Decimal>,
        ask_rate: &'a Option<rust_decimal::Decimal>,
        note: &'a Option<T4>,
        id: &'a T5,
    ) -> StringQuery<'c, 'a, 's, C, String, 9> {
        StringQuery {
            client,
            params: [
                quote_date,
                institution_id,
                term,
                term_sort,
                currency,
                bid_rate,
                ask_rate,
                note,
                id,
            ],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it.into(),
        }
    }
}
impl<
    'c,
    'a,
    's,
    C: GenericClient,
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::StringSql,
    T4: crate::StringSql,
    T5: crate::StringSql,
>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        DepositUpdateParams<T1, T2, T3, T4, T5>,
        StringQuery<'c, 'a, 's, C, String, 9>,
        C,
    > for DepositUpdateStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a DepositUpdateParams<T1, T2, T3, T4, T5>,
    ) -> StringQuery<'c, 'a, 's, C, String, 9> {
        self.bind(
            client,
            &params.quote_date,
            &params.institution_id,
            &params.term,
            &params.term_sort,
            &params.currency,
            &params.bid_rate,
            &params.ask_rate,
            &params.note,
            &params.id,
        )
    }
}
pub struct DepositDeleteStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn deposit_delete() -> DepositDeleteStmt {
    DepositDeleteStmt("DELETE FROM \"DepositQuote\" WHERE id = $1", None)
}
impl DepositDeleteStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub async fn bind<'c, 'a, 's, C: GenericClient, T1: crate::StringSql>(
        &'s self,
        client: &'c C,
        id: &'a T1,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[id]).await
    }
}
pub struct BondListStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn bond_list() -> BondListStmt {
    BondListStmt(
        "SELECT \"BondQuote\".id, \"BondQuote\".created_at, \"BondQuote\".quote_date, \"BondQuote\".institution_id, \"Institution\".name AS institution_name, \"Institution\".short_name AS institution_short_name, \"BondQuote\".bond_code, \"BondQuote\".bond_name, \"BondQuote\".currency, \"BondQuote\".maturity_date, \"BondQuote\".coupon_rate, \"BondQuote\".bid_yield, \"BondQuote\".ask_yield, \"BondQuote\".bid_price, \"BondQuote\".ask_price, \"BondQuote\".direction, \"BondQuote\".note FROM \"BondQuote\" INNER JOIN \"Institution\" ON \"Institution\".id = \"BondQuote\".institution_id WHERE (CAST($1 AS DATE) IS NULL OR \"BondQuote\".quote_date >= $1) AND (CAST($2 AS DATE) IS NULL OR \"BondQuote\".quote_date <= $2) AND (CAST($3 AS TEXT) IS NULL OR \"BondQuote\".institution_id = $3) ORDER BY \"BondQuote\".quote_date DESC, \"Institution\".name, \"BondQuote\".currency, \"BondQuote\".bond_code",
        None,
    )
}
impl BondListStmt {
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
        from: &'a Option<chrono::NaiveDate>,
        to: &'a Option<chrono::NaiveDate>,
        institution_id: &'a Option<T1>,
    ) -> BondListQuery<'c, 'a, 's, C, BondList, 3> {
        BondListQuery {
            client,
            params: [from, to, institution_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor:
                |row: &tokio_postgres::Row| -> Result<BondListBorrowed, tokio_postgres::Error> {
                    Ok(BondListBorrowed {
                        id: row.try_get(0)?,
                        created_at: row.try_get(1)?,
                        quote_date: row.try_get(2)?,
                        institution_id: row.try_get(3)?,
                        institution_name: row.try_get(4)?,
                        institution_short_name: row.try_get(5)?,
                        bond_code: row.try_get(6)?,
                        bond_name: row.try_get(7)?,
                        currency: row.try_get(8)?,
                        maturity_date: row.try_get(9)?,
                        coupon_rate: row.try_get(10)?,
                        bid_yield: row.try_get(11)?,
                        ask_yield: row.try_get(12)?,
                        bid_price: row.try_get(13)?,
                        ask_price: row.try_get(14)?,
                        direction: row.try_get(15)?,
                        note: row.try_get(16)?,
                    })
                },
            mapper: |it| BondList::from(it),
        }
    }
}
impl<'c, 'a, 's, C: GenericClient, T1: crate::StringSql>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        BondListParams<T1>,
        BondListQuery<'c, 'a, 's, C, BondList, 3>,
        C,
    > for BondListStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a BondListParams<T1>,
    ) -> BondListQuery<'c, 'a, 's, C, BondList, 3> {
        self.bind(client, &params.from, &params.to, &params.institution_id)
    }
}
pub struct BondPageStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn bond_page() -> BondPageStmt {
    BondPageStmt(
        "SELECT \"BondQuote\".id, \"BondQuote\".created_at, \"BondQuote\".quote_date, \"BondQuote\".institution_id, \"BondQuote\".bond_code, \"BondQuote\".bond_name, \"BondQuote\".currency, \"BondQuote\".maturity_date, \"BondQuote\".coupon_rate, \"BondQuote\".bid_yield, \"BondQuote\".ask_yield, \"BondQuote\".bid_price, \"BondQuote\".ask_price, \"BondQuote\".direction, \"BondQuote\".note FROM \"BondQuote\" WHERE \"BondQuote\".id = $1",
        None,
    )
}
impl BondPageStmt {
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
        id: &'a T1,
    ) -> BondPageQuery<'c, 'a, 's, C, BondPage, 1> {
        BondPageQuery {
            client,
            params: [id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor:
                |row: &tokio_postgres::Row| -> Result<BondPageBorrowed, tokio_postgres::Error> {
                    Ok(BondPageBorrowed {
                        id: row.try_get(0)?,
                        created_at: row.try_get(1)?,
                        quote_date: row.try_get(2)?,
                        institution_id: row.try_get(3)?,
                        bond_code: row.try_get(4)?,
                        bond_name: row.try_get(5)?,
                        currency: row.try_get(6)?,
                        maturity_date: row.try_get(7)?,
                        coupon_rate: row.try_get(8)?,
                        bid_yield: row.try_get(9)?,
                        ask_yield: row.try_get(10)?,
                        bid_price: row.try_get(11)?,
                        ask_price: row.try_get(12)?,
                        direction: row.try_get(13)?,
                        note: row.try_get(14)?,
                    })
                },
            mapper: |it| BondPage::from(it),
        }
    }
}
pub struct BondCreateStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn bond_create() -> BondCreateStmt {
    BondCreateStmt(
        "INSERT INTO \"BondQuote\" (id, quote_date, institution_id, bond_code, bond_name, currency, maturity_date, coupon_rate, bid_yield, ask_yield, bid_price, ask_price, direction, note) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14) RETURNING created_at",
        None,
    )
}
impl BondCreateStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub fn bind<
        'c,
        'a,
        's,
        C: GenericClient,
        T1: crate::StringSql,
        T2: crate::StringSql,
        T3: crate::StringSql,
        T4: crate::StringSql,
        T5: crate::StringSql,
        T6: crate::StringSql,
        T7: crate::StringSql,
    >(
        &'s self,
        client: &'c C,
        id: &'a T1,
        quote_date: &'a chrono::NaiveDate,
        institution_id: &'a T2,
        bond_code: &'a T3,
        bond_name: &'a Option<T4>,
        currency: &'a T5,
        maturity_date: &'a Option<chrono::NaiveDate>,
        coupon_rate: &'a Option<rust_decimal::Decimal>,
        bid_yield: &'a Option<rust_decimal::Decimal>,
        ask_yield: &'a Option<rust_decimal::Decimal>,
        bid_price: &'a Option<rust_decimal::Decimal>,
        ask_price: &'a Option<rust_decimal::Decimal>,
        direction: &'a Option<T6>,
        note: &'a Option<T7>,
    ) -> ChronoNaiveDateTimeQuery<'c, 'a, 's, C, chrono::NaiveDateTime, 14> {
        ChronoNaiveDateTimeQuery {
            client,
            params: [
                id,
                quote_date,
                institution_id,
                bond_code,
                bond_name,
                currency,
                maturity_date,
                coupon_rate,
                bid_yield,
                ask_yield,
                bid_price,
                ask_price,
                direction,
                note,
            ],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it,
        }
    }
}
impl<
    'c,
    'a,
    's,
    C: GenericClient,
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::StringSql,
    T4: crate::StringSql,
    T5: crate::StringSql,
    T6: crate::StringSql,
    T7: crate::StringSql,
>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        BondCreateParams<T1, T2, T3, T4, T5, T6, T7>,
        ChronoNaiveDateTimeQuery<'c, 'a, 's, C, chrono::NaiveDateTime, 14>,
        C,
    > for BondCreateStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a BondCreateParams<T1, T2, T3, T4, T5, T6, T7>,
    ) -> ChronoNaiveDateTimeQuery<'c, 'a, 's, C, chrono::NaiveDateTime, 14> {
        self.bind(
            client,
            &params.id,
            &params.quote_date,
            &params.institution_id,
            &params.bond_code,
            &params.bond_name,
            &params.currency,
            &params.maturity_date,
            &params.coupon_rate,
            &params.bid_yield,
            &params.ask_yield,
            &params.bid_price,
            &params.ask_price,
            &params.direction,
            &params.note,
        )
    }
}
pub struct BondUpdateStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn bond_update() -> BondUpdateStmt {
    BondUpdateStmt(
        "UPDATE \"BondQuote\" SET quote_date = $1, institution_id = $2, bond_code = $3, bond_name = $4, currency = $5, maturity_date = $6, coupon_rate = $7, bid_yield = $8, ask_yield = $9, bid_price = $10, ask_price = $11, direction = $12, note = $13 WHERE id = $14 RETURNING id",
        None,
    )
}
impl BondUpdateStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub fn bind<
        'c,
        'a,
        's,
        C: GenericClient,
        T1: crate::StringSql,
        T2: crate::StringSql,
        T3: crate::StringSql,
        T4: crate::StringSql,
        T5: crate::StringSql,
        T6: crate::StringSql,
        T7: crate::StringSql,
    >(
        &'s self,
        client: &'c C,
        quote_date: &'a chrono::NaiveDate,
        institution_id: &'a T1,
        bond_code: &'a T2,
        bond_name: &'a Option<T3>,
        currency: &'a T4,
        maturity_date: &'a Option<chrono::NaiveDate>,
        coupon_rate: &'a Option<rust_decimal::Decimal>,
        bid_yield: &'a Option<rust_decimal::Decimal>,
        ask_yield: &'a Option<rust_decimal::Decimal>,
        bid_price: &'a Option<rust_decimal::Decimal>,
        ask_price: &'a Option<rust_decimal::Decimal>,
        direction: &'a Option<T5>,
        note: &'a Option<T6>,
        id: &'a T7,
    ) -> StringQuery<'c, 'a, 's, C, String, 14> {
        StringQuery {
            client,
            params: [
                quote_date,
                institution_id,
                bond_code,
                bond_name,
                currency,
                maturity_date,
                coupon_rate,
                bid_yield,
                ask_yield,
                bid_price,
                ask_price,
                direction,
                note,
                id,
            ],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it.into(),
        }
    }
}
impl<
    'c,
    'a,
    's,
    C: GenericClient,
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::StringSql,
    T4: crate::StringSql,
    T5: crate::StringSql,
    T6: crate::StringSql,
    T7: crate::StringSql,
>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        BondUpdateParams<T1, T2, T3, T4, T5, T6, T7>,
        StringQuery<'c, 'a, 's, C, String, 14>,
        C,
    > for BondUpdateStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a BondUpdateParams<T1, T2, T3, T4, T5, T6, T7>,
    ) -> StringQuery<'c, 'a, 's, C, String, 14> {
        self.bind(
            client,
            &params.quote_date,
            &params.institution_id,
            &params.bond_code,
            &params.bond_name,
            &params.currency,
            &params.maturity_date,
            &params.coupon_rate,
            &params.bid_yield,
            &params.ask_yield,
            &params.bid_price,
            &params.ask_price,
            &params.direction,
            &params.note,
            &params.id,
        )
    }
}
pub struct BondDeleteStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn bond_delete() -> BondDeleteStmt {
    BondDeleteStmt("DELETE FROM \"BondQuote\" WHERE id = $1", None)
}
impl BondDeleteStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub async fn bind<'c, 'a, 's, C: GenericClient, T1: crate::StringSql>(
        &'s self,
        client: &'c C,
        id: &'a T1,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[id]).await
    }
}

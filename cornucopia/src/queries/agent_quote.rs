// This file was generated with `cornucopia`. Do not modify.

#[derive(Debug)]
pub struct DepositByDateParams<T1: crate::StringSql> {
    pub quote_date: Option<chrono::NaiveDate>,
    pub institution_name: Option<T1>,
}
#[derive(Debug)]
pub struct BondByDateParams<T1: crate::StringSql> {
    pub quote_date: Option<chrono::NaiveDate>,
    pub institution_name: Option<T1>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct DepositByDate {
    pub quote_date: chrono::NaiveDate,
    pub institution_name: String,
    pub institution_short_name: Option<String>,
    pub institution_cn_name: Option<String>,
    pub term: String,
    pub term_sort: i32,
    pub currency: String,
    pub bid_rate: Option<rust_decimal::Decimal>,
    pub ask_rate: Option<rust_decimal::Decimal>,
    pub note: Option<String>,
}
pub struct DepositByDateBorrowed<'a> {
    pub quote_date: chrono::NaiveDate,
    pub institution_name: &'a str,
    pub institution_short_name: Option<&'a str>,
    pub institution_cn_name: Option<&'a str>,
    pub term: &'a str,
    pub term_sort: i32,
    pub currency: &'a str,
    pub bid_rate: Option<rust_decimal::Decimal>,
    pub ask_rate: Option<rust_decimal::Decimal>,
    pub note: Option<&'a str>,
}
impl<'a> From<DepositByDateBorrowed<'a>> for DepositByDate {
    fn from(
        DepositByDateBorrowed {
            quote_date,
            institution_name,
            institution_short_name,
            institution_cn_name,
            term,
            term_sort,
            currency,
            bid_rate,
            ask_rate,
            note,
        }: DepositByDateBorrowed<'a>,
    ) -> Self {
        Self {
            quote_date,
            institution_name: institution_name.into(),
            institution_short_name: institution_short_name.map(|v| v.into()),
            institution_cn_name: institution_cn_name.map(|v| v.into()),
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
pub struct BondByDate {
    pub quote_date: chrono::NaiveDate,
    pub institution_name: String,
    pub institution_short_name: Option<String>,
    pub institution_cn_name: Option<String>,
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
pub struct BondByDateBorrowed<'a> {
    pub quote_date: chrono::NaiveDate,
    pub institution_name: &'a str,
    pub institution_short_name: Option<&'a str>,
    pub institution_cn_name: Option<&'a str>,
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
impl<'a> From<BondByDateBorrowed<'a>> for BondByDate {
    fn from(
        BondByDateBorrowed {
            quote_date,
            institution_name,
            institution_short_name,
            institution_cn_name,
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
        }: BondByDateBorrowed<'a>,
    ) -> Self {
        Self {
            quote_date,
            institution_name: institution_name.into(),
            institution_short_name: institution_short_name.map(|v| v.into()),
            institution_cn_name: institution_cn_name.map(|v| v.into()),
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
pub struct SearchBonds {
    pub quote_date: chrono::NaiveDate,
    pub institution_name: String,
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
}
pub struct SearchBondsBorrowed<'a> {
    pub quote_date: chrono::NaiveDate,
    pub institution_name: &'a str,
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
}
impl<'a> From<SearchBondsBorrowed<'a>> for SearchBonds {
    fn from(
        SearchBondsBorrowed {
            quote_date,
            institution_name,
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
        }: SearchBondsBorrowed<'a>,
    ) -> Self {
        Self {
            quote_date,
            institution_name: institution_name.into(),
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
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct Institutions {
    pub name: String,
    pub short_name: Option<String>,
    pub email: Option<String>,
    pub is_active: bool,
}
pub struct InstitutionsBorrowed<'a> {
    pub name: &'a str,
    pub short_name: Option<&'a str>,
    pub email: Option<&'a str>,
    pub is_active: bool,
}
impl<'a> From<InstitutionsBorrowed<'a>> for Institutions {
    fn from(
        InstitutionsBorrowed {
            name,
            short_name,
            email,
            is_active,
        }: InstitutionsBorrowed<'a>,
    ) -> Self {
        Self {
            name: name.into(),
            short_name: short_name.map(|v| v.into()),
            email: email.map(|v| v.into()),
            is_active,
        }
    }
}
use crate::client::async_::GenericClient;
use futures::{self, StreamExt, TryStreamExt};
pub struct ChronoNaiveDateQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<chrono::NaiveDate, tokio_postgres::Error>,
    mapper: fn(chrono::NaiveDate) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> ChronoNaiveDateQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(chrono::NaiveDate) -> R,
    ) -> ChronoNaiveDateQuery<'c, 'a, 's, C, R, N> {
        ChronoNaiveDateQuery {
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
pub struct DepositByDateQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<DepositByDateBorrowed, tokio_postgres::Error>,
    mapper: fn(DepositByDateBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> DepositByDateQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(DepositByDateBorrowed) -> R,
    ) -> DepositByDateQuery<'c, 'a, 's, C, R, N> {
        DepositByDateQuery {
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
pub struct BondByDateQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<BondByDateBorrowed, tokio_postgres::Error>,
    mapper: fn(BondByDateBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> BondByDateQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(BondByDateBorrowed) -> R,
    ) -> BondByDateQuery<'c, 'a, 's, C, R, N> {
        BondByDateQuery {
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
pub struct SearchBondsQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<SearchBondsBorrowed, tokio_postgres::Error>,
    mapper: fn(SearchBondsBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> SearchBondsQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(SearchBondsBorrowed) -> R,
    ) -> SearchBondsQuery<'c, 'a, 's, C, R, N> {
        SearchBondsQuery {
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
pub struct InstitutionsQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<InstitutionsBorrowed, tokio_postgres::Error>,
    mapper: fn(InstitutionsBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> InstitutionsQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(InstitutionsBorrowed) -> R,
    ) -> InstitutionsQuery<'c, 'a, 's, C, R, N> {
        InstitutionsQuery {
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
pub struct RecentQuoteDatesStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn recent_quote_dates() -> RecentQuoteDatesStmt {
    RecentQuoteDatesStmt(
        "SELECT DISTINCT quote_date FROM ( SELECT quote_date FROM \"DepositQuote\" UNION SELECT quote_date FROM \"BondQuote\" ) AS combined ORDER BY quote_date DESC LIMIT 14",
        None,
    )
}
impl RecentQuoteDatesStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub fn bind<'c, 'a, 's, C: GenericClient>(
        &'s self,
        client: &'c C,
    ) -> ChronoNaiveDateQuery<'c, 'a, 's, C, chrono::NaiveDate, 0> {
        ChronoNaiveDateQuery {
            client,
            params: [],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it,
        }
    }
}
pub struct DepositByDateStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn deposit_by_date() -> DepositByDateStmt {
    DepositByDateStmt(
        "SELECT \"DepositQuote\".quote_date, \"Institution\".name AS institution_name, \"Institution\".short_name AS institution_short_name, \"Institution\".cn_name AS institution_cn_name, \"DepositQuote\".term, \"DepositQuote\".term_sort, \"DepositQuote\".currency, \"DepositQuote\".bid_rate, \"DepositQuote\".ask_rate, \"DepositQuote\".note FROM \"DepositQuote\" INNER JOIN \"Institution\" ON \"Institution\".id = \"DepositQuote\".institution_id WHERE (CAST($1 AS DATE) IS NULL OR \"DepositQuote\".quote_date = CAST($1 AS DATE)) AND (CAST($2 AS TEXT) IS NULL OR \"Institution\".name ILIKE '%' || $2 || '%') ORDER BY \"DepositQuote\".quote_date DESC, \"Institution\".name, \"DepositQuote\".currency, \"DepositQuote\".term_sort",
        None,
    )
}
impl DepositByDateStmt {
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
        quote_date: &'a Option<chrono::NaiveDate>,
        institution_name: &'a Option<T1>,
    ) -> DepositByDateQuery<'c, 'a, 's, C, DepositByDate, 2> {
        DepositByDateQuery {
            client,
            params: [quote_date, institution_name],
            query: self.0,
            cached: self.1.as_ref(),
            extractor:
                |row: &tokio_postgres::Row| -> Result<DepositByDateBorrowed, tokio_postgres::Error> {
                    Ok(DepositByDateBorrowed {
                        quote_date: row.try_get(0)?,
                        institution_name: row.try_get(1)?,
                        institution_short_name: row.try_get(2)?,
                        institution_cn_name: row.try_get(3)?,
                        term: row.try_get(4)?,
                        term_sort: row.try_get(5)?,
                        currency: row.try_get(6)?,
                        bid_rate: row.try_get(7)?,
                        ask_rate: row.try_get(8)?,
                        note: row.try_get(9)?,
                    })
                },
            mapper: |it| DepositByDate::from(it),
        }
    }
}
impl<'c, 'a, 's, C: GenericClient, T1: crate::StringSql>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        DepositByDateParams<T1>,
        DepositByDateQuery<'c, 'a, 's, C, DepositByDate, 2>,
        C,
    > for DepositByDateStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a DepositByDateParams<T1>,
    ) -> DepositByDateQuery<'c, 'a, 's, C, DepositByDate, 2> {
        self.bind(client, &params.quote_date, &params.institution_name)
    }
}
pub struct BondByDateStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn bond_by_date() -> BondByDateStmt {
    BondByDateStmt(
        "SELECT \"BondQuote\".quote_date, \"Institution\".name AS institution_name, \"Institution\".short_name AS institution_short_name, \"Institution\".cn_name AS institution_cn_name, \"BondQuote\".bond_code, \"BondQuote\".bond_name, \"BondQuote\".currency, \"BondQuote\".maturity_date, \"BondQuote\".coupon_rate, \"BondQuote\".bid_yield, \"BondQuote\".ask_yield, \"BondQuote\".bid_price, \"BondQuote\".ask_price, \"BondQuote\".direction, \"BondQuote\".note FROM \"BondQuote\" INNER JOIN \"Institution\" ON \"Institution\".id = \"BondQuote\".institution_id WHERE (CAST($1 AS DATE) IS NULL OR \"BondQuote\".quote_date = CAST($1 AS DATE)) AND (CAST($2 AS TEXT) IS NULL OR \"Institution\".name ILIKE '%' || $2 || '%') ORDER BY \"BondQuote\".quote_date DESC, \"Institution\".name, \"BondQuote\".currency, \"BondQuote\".bond_code",
        None,
    )
}
impl BondByDateStmt {
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
        quote_date: &'a Option<chrono::NaiveDate>,
        institution_name: &'a Option<T1>,
    ) -> BondByDateQuery<'c, 'a, 's, C, BondByDate, 2> {
        BondByDateQuery {
            client,
            params: [quote_date, institution_name],
            query: self.0,
            cached: self.1.as_ref(),
            extractor:
                |row: &tokio_postgres::Row| -> Result<BondByDateBorrowed, tokio_postgres::Error> {
                    Ok(BondByDateBorrowed {
                        quote_date: row.try_get(0)?,
                        institution_name: row.try_get(1)?,
                        institution_short_name: row.try_get(2)?,
                        institution_cn_name: row.try_get(3)?,
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
            mapper: |it| BondByDate::from(it),
        }
    }
}
impl<'c, 'a, 's, C: GenericClient, T1: crate::StringSql>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        BondByDateParams<T1>,
        BondByDateQuery<'c, 'a, 's, C, BondByDate, 2>,
        C,
    > for BondByDateStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a BondByDateParams<T1>,
    ) -> BondByDateQuery<'c, 'a, 's, C, BondByDate, 2> {
        self.bind(client, &params.quote_date, &params.institution_name)
    }
}
pub struct SearchBondsStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn search_bonds() -> SearchBondsStmt {
    SearchBondsStmt(
        "SELECT \"BondQuote\".quote_date, \"Institution\".name AS institution_name, \"BondQuote\".bond_code, \"BondQuote\".bond_name, \"BondQuote\".currency, \"BondQuote\".maturity_date, \"BondQuote\".coupon_rate, \"BondQuote\".bid_yield, \"BondQuote\".ask_yield, \"BondQuote\".bid_price, \"BondQuote\".ask_price, \"BondQuote\".direction FROM \"BondQuote\" INNER JOIN \"Institution\" ON \"Institution\".id = \"BondQuote\".institution_id WHERE CAST($1 AS TEXT) IS NULL OR \"BondQuote\".bond_code ILIKE '%' || $1 || '%' OR \"BondQuote\".bond_name ILIKE '%' || $1 || '%' ORDER BY \"BondQuote\".quote_date DESC LIMIT 50",
        None,
    )
}
impl SearchBondsStmt {
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
        query: &'a Option<T1>,
    ) -> SearchBondsQuery<'c, 'a, 's, C, SearchBonds, 1> {
        SearchBondsQuery {
            client,
            params: [query],
            query: self.0,
            cached: self.1.as_ref(),
            extractor:
                |row: &tokio_postgres::Row| -> Result<SearchBondsBorrowed, tokio_postgres::Error> {
                    Ok(SearchBondsBorrowed {
                        quote_date: row.try_get(0)?,
                        institution_name: row.try_get(1)?,
                        bond_code: row.try_get(2)?,
                        bond_name: row.try_get(3)?,
                        currency: row.try_get(4)?,
                        maturity_date: row.try_get(5)?,
                        coupon_rate: row.try_get(6)?,
                        bid_yield: row.try_get(7)?,
                        ask_yield: row.try_get(8)?,
                        bid_price: row.try_get(9)?,
                        ask_price: row.try_get(10)?,
                        direction: row.try_get(11)?,
                    })
                },
            mapper: |it| SearchBonds::from(it),
        }
    }
}
pub struct InstitutionsStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn institutions() -> InstitutionsStmt {
    InstitutionsStmt(
        "SELECT \"Institution\".name, \"Institution\".short_name, \"Institution\".email, \"Institution\".is_active FROM \"Institution\" ORDER BY \"Institution\".name",
        None,
    )
}
impl InstitutionsStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub fn bind<'c, 'a, 's, C: GenericClient>(
        &'s self,
        client: &'c C,
    ) -> InstitutionsQuery<'c, 'a, 's, C, Institutions, 0> {
        InstitutionsQuery {
            client,
            params: [],
            query: self.0,
            cached: self.1.as_ref(),
            extractor:
                |row: &tokio_postgres::Row| -> Result<InstitutionsBorrowed, tokio_postgres::Error> {
                    Ok(InstitutionsBorrowed {
                        name: row.try_get(0)?,
                        short_name: row.try_get(1)?,
                        email: row.try_get(2)?,
                        is_active: row.try_get(3)?,
                    })
                },
            mapper: |it| Institutions::from(it),
        }
    }
}

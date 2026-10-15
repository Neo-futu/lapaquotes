// This file was generated with `cornucopia`. Do not modify.

#[derive(Debug)]
pub struct DepositsByDateParams<T1: crate::StringSql> {
    pub quote_date: Option<chrono::NaiveDate>,
    pub institution_id: Option<T1>,
}
#[derive(Debug)]
pub struct BondsByDateParams<T1: crate::StringSql> {
    pub quote_date: Option<chrono::NaiveDate>,
    pub institution_id: Option<T1>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Institutions {
    pub id: String,
    pub name: String,
    pub short_name: Option<String>,
}
pub struct InstitutionsBorrowed<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub short_name: Option<&'a str>,
}
impl<'a> From<InstitutionsBorrowed<'a>> for Institutions {
    fn from(
        InstitutionsBorrowed {
            id,
            name,
            short_name,
        }: InstitutionsBorrowed<'a>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            short_name: short_name.map(|v| v.into()),
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct DepositsByDate {
    pub id: String,
    pub quote_date: chrono::NaiveDate,
    pub institution_id: String,
    pub institution_name: String,
    pub institution_short_name: Option<String>,
    pub term: String,
    pub currency: String,
    pub bid_rate: Option<rust_decimal::Decimal>,
    pub ask_rate: Option<rust_decimal::Decimal>,
    pub note: Option<String>,
}
pub struct DepositsByDateBorrowed<'a> {
    pub id: &'a str,
    pub quote_date: chrono::NaiveDate,
    pub institution_id: &'a str,
    pub institution_name: &'a str,
    pub institution_short_name: Option<&'a str>,
    pub term: &'a str,
    pub currency: &'a str,
    pub bid_rate: Option<rust_decimal::Decimal>,
    pub ask_rate: Option<rust_decimal::Decimal>,
    pub note: Option<&'a str>,
}
impl<'a> From<DepositsByDateBorrowed<'a>> for DepositsByDate {
    fn from(
        DepositsByDateBorrowed {
            id,
            quote_date,
            institution_id,
            institution_name,
            institution_short_name,
            term,
            currency,
            bid_rate,
            ask_rate,
            note,
        }: DepositsByDateBorrowed<'a>,
    ) -> Self {
        Self {
            id: id.into(),
            quote_date,
            institution_id: institution_id.into(),
            institution_name: institution_name.into(),
            institution_short_name: institution_short_name.map(|v| v.into()),
            term: term.into(),
            currency: currency.into(),
            bid_rate,
            ask_rate,
            note: note.map(|v| v.into()),
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct BondsByDate {
    pub id: String,
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
pub struct BondsByDateBorrowed<'a> {
    pub id: &'a str,
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
impl<'a> From<BondsByDateBorrowed<'a>> for BondsByDate {
    fn from(
        BondsByDateBorrowed {
            id,
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
        }: BondsByDateBorrowed<'a>,
    ) -> Self {
        Self {
            id: id.into(),
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
use crate::client::async_::GenericClient;
use futures::{self, StreamExt, TryStreamExt};
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
pub struct OptionchronoNaiveDateQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<Option<chrono::NaiveDate>, tokio_postgres::Error>,
    mapper: fn(Option<chrono::NaiveDate>) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> OptionchronoNaiveDateQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(Option<chrono::NaiveDate>) -> R,
    ) -> OptionchronoNaiveDateQuery<'c, 'a, 's, C, R, N> {
        OptionchronoNaiveDateQuery {
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
pub struct DepositsByDateQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<DepositsByDateBorrowed, tokio_postgres::Error>,
    mapper: fn(DepositsByDateBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> DepositsByDateQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(DepositsByDateBorrowed) -> R,
    ) -> DepositsByDateQuery<'c, 'a, 's, C, R, N> {
        DepositsByDateQuery {
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
pub struct BondsByDateQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<BondsByDateBorrowed, tokio_postgres::Error>,
    mapper: fn(BondsByDateBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> BondsByDateQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(BondsByDateBorrowed) -> R,
    ) -> BondsByDateQuery<'c, 'a, 's, C, R, N> {
        BondsByDateQuery {
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
pub struct InstitutionsStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn institutions() -> InstitutionsStmt {
    InstitutionsStmt(
        "SELECT \"Institution\".id, \"Institution\".name, \"Institution\".short_name FROM \"Institution\" WHERE \"Institution\".is_active = true ORDER BY \"Institution\".name",
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
                        id: row.try_get(0)?,
                        name: row.try_get(1)?,
                        short_name: row.try_get(2)?,
                    })
                },
            mapper: |it| Institutions::from(it),
        }
    }
}
pub struct LatestDepositDateStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn latest_deposit_date() -> LatestDepositDateStmt {
    LatestDepositDateStmt(
        "SELECT MAX(\"DepositQuote\".quote_date) AS quote_date FROM \"DepositQuote\"",
        None,
    )
}
impl LatestDepositDateStmt {
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
    ) -> OptionchronoNaiveDateQuery<'c, 'a, 's, C, Option<chrono::NaiveDate>, 0> {
        OptionchronoNaiveDateQuery {
            client,
            params: [],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it,
        }
    }
}
pub struct LatestBondDateStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn latest_bond_date() -> LatestBondDateStmt {
    LatestBondDateStmt(
        "SELECT MAX(\"BondQuote\".quote_date) AS quote_date FROM \"BondQuote\"",
        None,
    )
}
impl LatestBondDateStmt {
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
    ) -> OptionchronoNaiveDateQuery<'c, 'a, 's, C, Option<chrono::NaiveDate>, 0> {
        OptionchronoNaiveDateQuery {
            client,
            params: [],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it,
        }
    }
}
pub struct DepositsByDateStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn deposits_by_date() -> DepositsByDateStmt {
    DepositsByDateStmt(
        "SELECT \"DepositQuote\".id, \"DepositQuote\".quote_date, \"DepositQuote\".institution_id, \"Institution\".name AS institution_name, \"Institution\".short_name AS institution_short_name, \"DepositQuote\".term, \"DepositQuote\".currency, \"DepositQuote\".bid_rate, \"DepositQuote\".ask_rate, \"DepositQuote\".note FROM \"DepositQuote\" INNER JOIN \"Institution\" ON \"Institution\".id = \"DepositQuote\".institution_id WHERE (CAST($1 AS DATE) IS NULL OR \"DepositQuote\".quote_date = CAST($1 AS DATE)) AND (CAST($2 AS TEXT) IS NULL OR \"DepositQuote\".institution_id = $2) ORDER BY \"DepositQuote\".quote_date DESC, \"Institution\".name, \"DepositQuote\".currency, \"DepositQuote\".term_sort",
        None,
    )
}
impl DepositsByDateStmt {
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
        institution_id: &'a Option<T1>,
    ) -> DepositsByDateQuery<'c, 'a, 's, C, DepositsByDate, 2> {
        DepositsByDateQuery {
            client,
            params: [quote_date, institution_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<DepositsByDateBorrowed, tokio_postgres::Error> {
                Ok(DepositsByDateBorrowed {
                    id: row.try_get(0)?,
                    quote_date: row.try_get(1)?,
                    institution_id: row.try_get(2)?,
                    institution_name: row.try_get(3)?,
                    institution_short_name: row.try_get(4)?,
                    term: row.try_get(5)?,
                    currency: row.try_get(6)?,
                    bid_rate: row.try_get(7)?,
                    ask_rate: row.try_get(8)?,
                    note: row.try_get(9)?,
                })
            },
            mapper: |it| DepositsByDate::from(it),
        }
    }
}
impl<'c, 'a, 's, C: GenericClient, T1: crate::StringSql>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        DepositsByDateParams<T1>,
        DepositsByDateQuery<'c, 'a, 's, C, DepositsByDate, 2>,
        C,
    > for DepositsByDateStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a DepositsByDateParams<T1>,
    ) -> DepositsByDateQuery<'c, 'a, 's, C, DepositsByDate, 2> {
        self.bind(client, &params.quote_date, &params.institution_id)
    }
}
pub struct BondsByDateStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn bonds_by_date() -> BondsByDateStmt {
    BondsByDateStmt(
        "SELECT \"BondQuote\".id, \"BondQuote\".quote_date, \"BondQuote\".institution_id, \"Institution\".name AS institution_name, \"Institution\".short_name AS institution_short_name, \"BondQuote\".bond_code, \"BondQuote\".bond_name, \"BondQuote\".currency, \"BondQuote\".maturity_date, \"BondQuote\".coupon_rate, \"BondQuote\".bid_yield, \"BondQuote\".ask_yield, \"BondQuote\".bid_price, \"BondQuote\".ask_price, \"BondQuote\".direction, \"BondQuote\".note FROM \"BondQuote\" INNER JOIN \"Institution\" ON \"Institution\".id = \"BondQuote\".institution_id WHERE (CAST($1 AS DATE) IS NULL OR \"BondQuote\".quote_date = CAST($1 AS DATE)) AND (CAST($2 AS TEXT) IS NULL OR \"BondQuote\".institution_id = $2) ORDER BY \"BondQuote\".quote_date DESC, \"Institution\".name, \"BondQuote\".currency, \"BondQuote\".bond_code",
        None,
    )
}
impl BondsByDateStmt {
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
        institution_id: &'a Option<T1>,
    ) -> BondsByDateQuery<'c, 'a, 's, C, BondsByDate, 2> {
        BondsByDateQuery {
            client,
            params: [quote_date, institution_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor:
                |row: &tokio_postgres::Row| -> Result<BondsByDateBorrowed, tokio_postgres::Error> {
                    Ok(BondsByDateBorrowed {
                        id: row.try_get(0)?,
                        quote_date: row.try_get(1)?,
                        institution_id: row.try_get(2)?,
                        institution_name: row.try_get(3)?,
                        institution_short_name: row.try_get(4)?,
                        bond_code: row.try_get(5)?,
                        bond_name: row.try_get(6)?,
                        currency: row.try_get(7)?,
                        maturity_date: row.try_get(8)?,
                        coupon_rate: row.try_get(9)?,
                        bid_yield: row.try_get(10)?,
                        ask_yield: row.try_get(11)?,
                        bid_price: row.try_get(12)?,
                        ask_price: row.try_get(13)?,
                        direction: row.try_get(14)?,
                        note: row.try_get(15)?,
                    })
                },
            mapper: |it| BondsByDate::from(it),
        }
    }
}
impl<'c, 'a, 's, C: GenericClient, T1: crate::StringSql>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        BondsByDateParams<T1>,
        BondsByDateQuery<'c, 'a, 's, C, BondsByDate, 2>,
        C,
    > for BondsByDateStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a BondsByDateParams<T1>,
    ) -> BondsByDateQuery<'c, 'a, 's, C, BondsByDate, 2> {
        self.bind(client, &params.quote_date, &params.institution_id)
    }
}

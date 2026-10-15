// This file was generated with `cornucopia`. Do not modify.

#[derive(Debug)]
pub struct CreateParams<
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::StringSql,
    T4: crate::StringSql,
> {
    pub id: T1,
    pub name: T2,
    pub short_name: Option<T3>,
    pub email: Option<T4>,
    pub is_active: bool,
}
#[derive(Debug)]
pub struct UpdateParams<
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::StringSql,
    T4: crate::StringSql,
> {
    pub name: T1,
    pub short_name: Option<T2>,
    pub email: Option<T3>,
    pub is_active: bool,
    pub id: T4,
}
#[derive(Debug, Clone, PartialEq)]
pub struct List {
    pub id: String,
    pub created_at: chrono::NaiveDateTime,
    pub name: String,
    pub short_name: Option<String>,
    pub email: Option<String>,
    pub is_active: bool,
}
pub struct ListBorrowed<'a> {
    pub id: &'a str,
    pub created_at: chrono::NaiveDateTime,
    pub name: &'a str,
    pub short_name: Option<&'a str>,
    pub email: Option<&'a str>,
    pub is_active: bool,
}
impl<'a> From<ListBorrowed<'a>> for List {
    fn from(
        ListBorrowed {
            id,
            created_at,
            name,
            short_name,
            email,
            is_active,
        }: ListBorrowed<'a>,
    ) -> Self {
        Self {
            id: id.into(),
            created_at,
            name: name.into(),
            short_name: short_name.map(|v| v.into()),
            email: email.map(|v| v.into()),
            is_active,
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct Page {
    pub id: String,
    pub created_at: chrono::NaiveDateTime,
    pub name: String,
    pub short_name: Option<String>,
    pub email: Option<String>,
    pub is_active: bool,
}
pub struct PageBorrowed<'a> {
    pub id: &'a str,
    pub created_at: chrono::NaiveDateTime,
    pub name: &'a str,
    pub short_name: Option<&'a str>,
    pub email: Option<&'a str>,
    pub is_active: bool,
}
impl<'a> From<PageBorrowed<'a>> for Page {
    fn from(
        PageBorrowed {
            id,
            created_at,
            name,
            short_name,
            email,
            is_active,
        }: PageBorrowed<'a>,
    ) -> Self {
        Self {
            id: id.into(),
            created_at,
            name: name.into(),
            short_name: short_name.map(|v| v.into()),
            email: email.map(|v| v.into()),
            is_active,
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct FindByName {
    pub id: String,
    pub name: String,
}
pub struct FindByNameBorrowed<'a> {
    pub id: &'a str,
    pub name: &'a str,
}
impl<'a> From<FindByNameBorrowed<'a>> for FindByName {
    fn from(FindByNameBorrowed { id, name }: FindByNameBorrowed<'a>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
        }
    }
}
use crate::client::async_::GenericClient;
use futures::{self, StreamExt, TryStreamExt};
pub struct ListQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<ListBorrowed, tokio_postgres::Error>,
    mapper: fn(ListBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> ListQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(self, mapper: fn(ListBorrowed) -> R) -> ListQuery<'c, 'a, 's, C, R, N> {
        ListQuery {
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
pub struct PageQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<PageBorrowed, tokio_postgres::Error>,
    mapper: fn(PageBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> PageQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(self, mapper: fn(PageBorrowed) -> R) -> PageQuery<'c, 'a, 's, C, R, N> {
        PageQuery {
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
pub struct FindByNameQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<FindByNameBorrowed, tokio_postgres::Error>,
    mapper: fn(FindByNameBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> FindByNameQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(FindByNameBorrowed) -> R,
    ) -> FindByNameQuery<'c, 'a, 's, C, R, N> {
        FindByNameQuery {
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
pub struct I64Query<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<i64, tokio_postgres::Error>,
    mapper: fn(i64) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> I64Query<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(self, mapper: fn(i64) -> R) -> I64Query<'c, 'a, 's, C, R, N> {
        I64Query {
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
pub struct ListStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn list() -> ListStmt {
    ListStmt(
        "SELECT id, created_at, name, short_name, email, is_active FROM \"Institution\" ORDER BY name",
        None,
    )
}
impl ListStmt {
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
    ) -> ListQuery<'c, 'a, 's, C, List, 0> {
        ListQuery {
            client,
            params: [],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row: &tokio_postgres::Row| -> Result<ListBorrowed, tokio_postgres::Error> {
                Ok(ListBorrowed {
                    id: row.try_get(0)?,
                    created_at: row.try_get(1)?,
                    name: row.try_get(2)?,
                    short_name: row.try_get(3)?,
                    email: row.try_get(4)?,
                    is_active: row.try_get(5)?,
                })
            },
            mapper: |it| List::from(it),
        }
    }
}
pub struct PageStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn page() -> PageStmt {
    PageStmt(
        "SELECT id, created_at, name, short_name, email, is_active FROM \"Institution\" WHERE id = $1",
        None,
    )
}
impl PageStmt {
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
    ) -> PageQuery<'c, 'a, 's, C, Page, 1> {
        PageQuery {
            client,
            params: [id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row: &tokio_postgres::Row| -> Result<PageBorrowed, tokio_postgres::Error> {
                Ok(PageBorrowed {
                    id: row.try_get(0)?,
                    created_at: row.try_get(1)?,
                    name: row.try_get(2)?,
                    short_name: row.try_get(3)?,
                    email: row.try_get(4)?,
                    is_active: row.try_get(5)?,
                })
            },
            mapper: |it| Page::from(it),
        }
    }
}
pub struct FindByNameStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn find_by_name() -> FindByNameStmt {
    FindByNameStmt("SELECT id, name FROM \"Institution\" WHERE name = $1", None)
}
impl FindByNameStmt {
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
        name: &'a T1,
    ) -> FindByNameQuery<'c, 'a, 's, C, FindByName, 1> {
        FindByNameQuery {
            client,
            params: [name],
            query: self.0,
            cached: self.1.as_ref(),
            extractor:
                |row: &tokio_postgres::Row| -> Result<FindByNameBorrowed, tokio_postgres::Error> {
                    Ok(FindByNameBorrowed {
                        id: row.try_get(0)?,
                        name: row.try_get(1)?,
                    })
                },
            mapper: |it| FindByName::from(it),
        }
    }
}
pub struct CreateStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn create() -> CreateStmt {
    CreateStmt(
        "INSERT INTO \"Institution\" (id, name, short_name, email, is_active) VALUES ($1, $2, $3, $4, $5) RETURNING created_at",
        None,
    )
}
impl CreateStmt {
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
    >(
        &'s self,
        client: &'c C,
        id: &'a T1,
        name: &'a T2,
        short_name: &'a Option<T3>,
        email: &'a Option<T4>,
        is_active: &'a bool,
    ) -> ChronoNaiveDateTimeQuery<'c, 'a, 's, C, chrono::NaiveDateTime, 5> {
        ChronoNaiveDateTimeQuery {
            client,
            params: [id, name, short_name, email, is_active],
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
>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        CreateParams<T1, T2, T3, T4>,
        ChronoNaiveDateTimeQuery<'c, 'a, 's, C, chrono::NaiveDateTime, 5>,
        C,
    > for CreateStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a CreateParams<T1, T2, T3, T4>,
    ) -> ChronoNaiveDateTimeQuery<'c, 'a, 's, C, chrono::NaiveDateTime, 5> {
        self.bind(
            client,
            &params.id,
            &params.name,
            &params.short_name,
            &params.email,
            &params.is_active,
        )
    }
}
pub struct UpdateStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn update() -> UpdateStmt {
    UpdateStmt(
        "UPDATE \"Institution\" SET name = $1, short_name = $2, email = $3, is_active = $4 WHERE id = $5 RETURNING id",
        None,
    )
}
impl UpdateStmt {
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
    >(
        &'s self,
        client: &'c C,
        name: &'a T1,
        short_name: &'a Option<T2>,
        email: &'a Option<T3>,
        is_active: &'a bool,
        id: &'a T4,
    ) -> StringQuery<'c, 'a, 's, C, String, 5> {
        StringQuery {
            client,
            params: [name, short_name, email, is_active, id],
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
>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        UpdateParams<T1, T2, T3, T4>,
        StringQuery<'c, 'a, 's, C, String, 5>,
        C,
    > for UpdateStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a UpdateParams<T1, T2, T3, T4>,
    ) -> StringQuery<'c, 'a, 's, C, String, 5> {
        self.bind(
            client,
            &params.name,
            &params.short_name,
            &params.email,
            &params.is_active,
            &params.id,
        )
    }
}
pub struct DeleteStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn delete() -> DeleteStmt {
    DeleteStmt("DELETE FROM \"Institution\" WHERE id = $1", None)
}
impl DeleteStmt {
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
pub struct CountQuotesStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn count_quotes() -> CountQuotesStmt {
    CountQuotesStmt(
        "SELECT (SELECT COUNT(*) FROM \"DepositQuote\" WHERE institution_id = $1) + (SELECT COUNT(*) FROM \"BondQuote\" WHERE institution_id = $1) AS count",
        None,
    )
}
impl CountQuotesStmt {
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
    ) -> I64Query<'c, 'a, 's, C, i64, 1> {
        I64Query {
            client,
            params: [id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it,
        }
    }
}

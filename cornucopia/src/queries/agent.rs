// This file was generated with `cornucopia`. Do not modify.

#[derive(Debug, Clone, PartialEq)]
pub struct PostBySlug {
    pub id: String,
    pub slug: String,
    pub h1: String,
    pub meta_title: String,
    pub meta_description: String,
    pub category_slug: String,
    pub content_id: String,
    pub content_json: String,
}
pub struct PostBySlugBorrowed<'a> {
    pub id: &'a str,
    pub slug: &'a str,
    pub h1: &'a str,
    pub meta_title: &'a str,
    pub meta_description: &'a str,
    pub category_slug: &'a str,
    pub content_id: &'a str,
    pub content_json: &'a str,
}
impl<'a> From<PostBySlugBorrowed<'a>> for PostBySlug {
    fn from(
        PostBySlugBorrowed {
            id,
            slug,
            h1,
            meta_title,
            meta_description,
            category_slug,
            content_id,
            content_json,
        }: PostBySlugBorrowed<'a>,
    ) -> Self {
        Self {
            id: id.into(),
            slug: slug.into(),
            h1: h1.into(),
            meta_title: meta_title.into(),
            meta_description: meta_description.into(),
            category_slug: category_slug.into(),
            content_id: content_id.into(),
            content_json: content_json.into(),
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct PostById {
    pub id: String,
    pub slug: String,
    pub h1: String,
    pub meta_title: String,
    pub meta_description: String,
    pub category_slug: String,
    pub content_id: String,
    pub content_json: String,
}
pub struct PostByIdBorrowed<'a> {
    pub id: &'a str,
    pub slug: &'a str,
    pub h1: &'a str,
    pub meta_title: &'a str,
    pub meta_description: &'a str,
    pub category_slug: &'a str,
    pub content_id: &'a str,
    pub content_json: &'a str,
}
impl<'a> From<PostByIdBorrowed<'a>> for PostById {
    fn from(
        PostByIdBorrowed {
            id,
            slug,
            h1,
            meta_title,
            meta_description,
            category_slug,
            content_id,
            content_json,
        }: PostByIdBorrowed<'a>,
    ) -> Self {
        Self {
            id: id.into(),
            slug: slug.into(),
            h1: h1.into(),
            meta_title: meta_title.into(),
            meta_description: meta_description.into(),
            category_slug: category_slug.into(),
            content_id: content_id.into(),
            content_json: content_json.into(),
        }
    }
}
use crate::client::async_::GenericClient;
use futures::{self, StreamExt, TryStreamExt};
pub struct PostBySlugQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<PostBySlugBorrowed, tokio_postgres::Error>,
    mapper: fn(PostBySlugBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> PostBySlugQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(PostBySlugBorrowed) -> R,
    ) -> PostBySlugQuery<'c, 'a, 's, C, R, N> {
        PostBySlugQuery {
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
pub struct PostByIdQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<PostByIdBorrowed, tokio_postgres::Error>,
    mapper: fn(PostByIdBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> PostByIdQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(self, mapper: fn(PostByIdBorrowed) -> R) -> PostByIdQuery<'c, 'a, 's, C, R, N> {
        PostByIdQuery {
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
pub struct PostBySlugStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn post_by_slug() -> PostBySlugStmt {
    PostBySlugStmt(
        "SELECT \"Post\".id, \"Post\".slug, \"Post\".h1, \"Post\".meta_title, \"Post\".meta_description, \"PostCategory\".slug AS category_slug, \"Content\".id AS content_id, \"Content\".json AS content_json FROM \"Post\" INNER JOIN \"Content\" ON \"Content\".id = \"Post\".content_id INNER JOIN \"PostCategory\" ON \"PostCategory\".id = \"Post\".category_id WHERE \"Post\".slug = $1 AND \"Post\".publish_at < NOW() LIMIT 1",
        None,
    )
}
impl PostBySlugStmt {
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
        slug: &'a T1,
    ) -> PostBySlugQuery<'c, 'a, 's, C, PostBySlug, 1> {
        PostBySlugQuery {
            client,
            params: [slug],
            query: self.0,
            cached: self.1.as_ref(),
            extractor:
                |row: &tokio_postgres::Row| -> Result<PostBySlugBorrowed, tokio_postgres::Error> {
                    Ok(PostBySlugBorrowed {
                        id: row.try_get(0)?,
                        slug: row.try_get(1)?,
                        h1: row.try_get(2)?,
                        meta_title: row.try_get(3)?,
                        meta_description: row.try_get(4)?,
                        category_slug: row.try_get(5)?,
                        content_id: row.try_get(6)?,
                        content_json: row.try_get(7)?,
                    })
                },
            mapper: |it| PostBySlug::from(it),
        }
    }
}
pub struct PostByIdStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn post_by_id() -> PostByIdStmt {
    PostByIdStmt(
        "SELECT \"Post\".id, \"Post\".slug, \"Post\".h1, \"Post\".meta_title, \"Post\".meta_description, \"PostCategory\".slug AS category_slug, \"Content\".id AS content_id, \"Content\".json AS content_json FROM \"Post\" INNER JOIN \"Content\" ON \"Content\".id = \"Post\".content_id INNER JOIN \"PostCategory\" ON \"PostCategory\".id = \"Post\".category_id WHERE \"Post\".id = $1 AND \"Post\".publish_at < NOW()",
        None,
    )
}
impl PostByIdStmt {
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
    ) -> PostByIdQuery<'c, 'a, 's, C, PostById, 1> {
        PostByIdQuery {
            client,
            params: [id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor:
                |row: &tokio_postgres::Row| -> Result<PostByIdBorrowed, tokio_postgres::Error> {
                    Ok(PostByIdBorrowed {
                        id: row.try_get(0)?,
                        slug: row.try_get(1)?,
                        h1: row.try_get(2)?,
                        meta_title: row.try_get(3)?,
                        meta_description: row.try_get(4)?,
                        category_slug: row.try_get(5)?,
                        content_id: row.try_get(6)?,
                        content_json: row.try_get(7)?,
                    })
                },
            mapper: |it| PostById::from(it),
        }
    }
}

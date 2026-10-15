use cornucopia::{deadpool_postgres::Pool, queries, tokio_postgres};
use serde::Serialize;
use thiserror::Error;

use content::content_json_to_html;

/// Full grounding for one published post.
#[derive(Debug, Clone, Serialize)]
pub struct PostGrounding {
    pub id: String,
    pub slug: String,
    pub category_slug: String,
    pub h1: String,
    pub meta_title: String,
    pub meta_description: String,
    /// Plain text derived from Slate JSON.
    pub content_text: String,
}

/// Compact listing row for tools / search.
#[derive(Debug, Clone, Serialize)]
pub struct PostSummary {
    pub id: String,
    pub slug: String,
    pub category_slug: String,
    pub h1: String,
}

#[derive(Debug, Error)]
pub enum ContextError {
    #[error("database pool error: {0}")]
    Pool(String),
    #[error("database query error: {0}")]
    Query(#[from] tokio_postgres::Error),
    #[error("post not found or unpublished")]
    NotFound,
}

pub async fn load_post_grounding(
    pool: &Pool,
    post_id: Option<&str>,
    slug: Option<&str>,
) -> Result<Option<PostGrounding>, ContextError> {
    if post_id.is_none() && slug.is_none() {
        return Ok(None);
    }

    let client = pool
        .get()
        .await
        .map_err(|e| ContextError::Pool(e.to_string()))?;

    let grounding = if let Some(id) = post_id {
        queries::agent::post_by_id()
            .bind(&client, &id)
            .opt()
            .await?
            .map(|row| row_to_grounding(row.id, row.slug, row.h1, row.meta_title, row.meta_description, row.category_slug, row.content_json))
    } else if let Some(slug) = slug {
        queries::agent::post_by_slug()
            .bind(&client, &slug)
            .opt()
            .await?
            .map(|row| row_to_grounding(row.id, row.slug, row.h1, row.meta_title, row.meta_description, row.category_slug, row.content_json))
    } else {
        None
    };

    let Some(grounding) = grounding else {
        return Err(ContextError::NotFound);
    };

    Ok(Some(grounding))
}

pub async fn list_published_posts(pool: &Pool) -> Result<Vec<PostSummary>, ContextError> {
    let client = pool
        .get()
        .await
        .map_err(|e| ContextError::Pool(e.to_string()))?;

    let rows = queries::post::list().bind(&client).all().await?;
    Ok(rows
        .into_iter()
        .map(|r| PostSummary {
            id: r.id,
            slug: r.slug,
            category_slug: r.category_slug,
            h1: r.h1,
        })
        .collect())
}

/// Case-insensitive match against slug / h1 / category of published posts.
pub async fn search_published_posts(
    pool: &Pool,
    query: &str,
) -> Result<Vec<PostSummary>, ContextError> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return Ok(vec![]);
    }
    let all = list_published_posts(pool).await?;
    Ok(all
        .into_iter()
        .filter(|p| {
            p.slug.to_lowercase().contains(&q)
                || p.h1.to_lowercase().contains(&q)
                || p.category_slug.to_lowercase().contains(&q)
        })
        .collect())
}

#[allow(clippy::too_many_arguments)]
fn row_to_grounding(
    id: String,
    slug: String,
    h1: String,
    meta_title: String,
    meta_description: String,
    category_slug: String,
    content_json: String,
) -> PostGrounding {
    let html = content_json_to_html(&content_json);
    PostGrounding {
        id,
        slug,
        category_slug,
        h1,
        meta_title,
        meta_description,
        content_text: strip_tags(&html),
    }
}

fn strip_tags(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

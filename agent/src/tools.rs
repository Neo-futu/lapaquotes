//! Post tools for the Rig agent — pattern from
//! `examples/agent_with_tools` (DynamicTool + JSON schemas).

use cornucopia::deadpool_postgres::Pool;
use rig::message::{EmptyToolName, ToolName};
use rig::tool::{DynamicTool, ToolOutput};
use serde::Deserialize;
use serde_json::json;

use crate::context::{load_post_grounding, list_published_posts, search_published_posts};

#[derive(Deserialize)]
struct SlugArgs {
    slug: String,
}

#[derive(Deserialize)]
struct SearchArgs {
    query: String,
}

#[derive(Deserialize)]
struct EmptyArgs {}

/// Runtime tools the model must use to read published posts from Postgres.
pub fn post_tools(pool: Pool) -> Result<Vec<DynamicTool>, EmptyToolName> {
    let pool_get = pool.clone();
    let pool_list = pool.clone();
    let pool_search = pool;

    Ok(vec![
        DynamicTool::new(
            ToolName::new("get_post_by_slug")?,
            "Fetch one published post by its slug, including title, meta and full content text. \
             Use this before answering questions about a specific article.",
            json!({
                "type": "object",
                "properties": {
                    "slug": {
                        "type": "string",
                        "description": "The post slug, e.g. hello-world"
                    }
                },
                "required": ["slug"]
            }),
            move |args| {
                let pool = pool_get.clone();
                Box::pin(async move {
                    let args: SlugArgs = serde_json::from_value(args).map_err(|error| {
                        rig::tool::ToolExecutionError::invalid_args(error.to_string())
                            .with_source(error)
                    })?;
                    match load_post_grounding(&pool, None, Some(args.slug.as_str())).await {
                        Ok(Some(post)) => Ok(ToolOutput::json(
                            serde_json::to_value(post).unwrap_or_else(|e| {
                                json!({ "error": e.to_string() })
                            }),
                        )),
                        Ok(None) => Ok(ToolOutput::json(json!({
                            "error": "no slug provided"
                        }))),
                        Err(e) => Ok(ToolOutput::json(json!({
                            "error": e.to_string()
                        }))),
                    }
                })
            },
        ),
        DynamicTool::new(
            ToolName::new("list_published_posts")?,
            "List all published posts (id, slug, category, title). \
             Use when the user asks what articles exist or does not give a slug.",
            json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }),
            move |_args| {
                let pool = pool_list.clone();
                Box::pin(async move {
                    let _args: EmptyArgs = EmptyArgs {};
                    match list_published_posts(&pool).await {
                        Ok(posts) => Ok(ToolOutput::json(
                            serde_json::to_value(posts).unwrap_or_else(|e| {
                                json!({ "error": e.to_string() })
                            }),
                        )),
                        Err(e) => Ok(ToolOutput::json(json!({
                            "error": e.to_string()
                        }))),
                    }
                })
            },
        ),
        DynamicTool::new(
            ToolName::new("search_published_posts")?,
            "Search published posts by keyword against slug, title (h1) and category. \
             Returns matching summaries; then call get_post_by_slug for full content.",
            json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search keyword"
                    }
                },
                "required": ["query"]
            }),
            move |args| {
                let pool = pool_search.clone();
                Box::pin(async move {
                    let args: SearchArgs = serde_json::from_value(args).map_err(|error| {
                        rig::tool::ToolExecutionError::invalid_args(error.to_string())
                            .with_source(error)
                    })?;
                    match search_published_posts(&pool, &args.query).await {
                        Ok(posts) => Ok(ToolOutput::json(
                            serde_json::to_value(posts).unwrap_or_else(|e| {
                                json!({ "error": e.to_string() })
                            }),
                        )),
                        Err(e) => Ok(ToolOutput::json(json!({
                            "error": e.to_string()
                        }))),
                    }
                })
            },
        ),
    ])
}

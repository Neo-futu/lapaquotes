//! HTTP surface for BI layer: `/agent/bi/*`.

use axum::{
    Json, Router,
    extract::{FromRef, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use chrono::NaiveDate;
use cornucopia::deadpool_postgres::Pool;
use serde::Deserialize;
use serde_json::json;

use crate::api::ErrorBody;

use super::service::BiService;

pub fn bi_router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    Pool: FromRef<S>,
{
    Router::new()
        .route("/catalog", get(catalog_get::<S>))
        .route("/catalog/refresh", post(catalog_refresh::<S>))
        .route("/migrate", post(migrate::<S>))
        .route("/sync", post(sync::<S>))
        .route("/verify", post(verify::<S>))
        .route("/facts", get(facts_get::<S>))
        .route("/jobs", get(jobs_get::<S>))
        .route("/dashboards", get(dashboards_get::<S>))
        .route("/dashboards/ensure", post(dashboards_ensure::<S>))
}

fn svc<S>(state: &S) -> BiService
where
    Pool: FromRef<S>,
{
    BiService::new(Pool::from_ref(state))
}

async fn catalog_get<S>(State(state): State<S>) -> impl IntoResponse
where
    S: Send + Sync,
    Pool: FromRef<S>,
{
    match svc(&state).list_catalog().await {
        Ok(tables) => (StatusCode::OK, Json(json!({ "tables": tables }))).into_response(),
        Err(e) => err(e),
    }
}

async fn catalog_refresh<S>(State(state): State<S>) -> impl IntoResponse
where
    S: Send + Sync,
    Pool: FromRef<S>,
{
    match svc(&state).refresh_catalog().await {
        Ok(tables) => (StatusCode::OK, Json(json!({ "tables": tables }))).into_response(),
        Err(e) => err(e),
    }
}

async fn migrate<S>(State(state): State<S>) -> impl IntoResponse
where
    S: Send + Sync,
    Pool: FromRef<S>,
{
    match svc(&state).migrate_bitable().await {
        Ok(report) => (StatusCode::OK, Json(report)).into_response(),
        Err(e) => err(e),
    }
}

#[derive(Debug, Deserialize)]
struct SyncBody {
    #[serde(default)]
    date: Option<String>,
}

async fn sync<S>(State(state): State<S>, Json(body): Json<SyncBody>) -> impl IntoResponse
where
    S: Send + Sync,
    Pool: FromRef<S>,
{
    let date = match parse_date_opt(body.date.as_deref()) {
        Ok(d) => d,
        Err(e) => return bad(e),
    };
    match svc(&state).sync_day(date).await {
        Ok(report) => (StatusCode::OK, Json(report)).into_response(),
        Err(e) => err(e),
    }
}

async fn verify<S>(State(state): State<S>) -> impl IntoResponse
where
    S: Send + Sync,
    Pool: FromRef<S>,
{
    match svc(&state).verify().await {
        Ok(report) => {
            let status = if report.ok {
                StatusCode::OK
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            (status, Json(report)).into_response()
        }
        Err(e) => err(e),
    }
}

#[derive(Debug, Deserialize)]
struct DateQuery {
    #[serde(default)]
    date: Option<String>,
}

async fn facts_get<S>(
    State(state): State<S>,
    Query(q): Query<DateQuery>,
) -> impl IntoResponse
where
    S: Send + Sync,
    Pool: FromRef<S>,
{
    let date = match parse_date_opt(q.date.as_deref()) {
        Ok(d) => d,
        Err(e) => return bad(e),
    };
    match svc(&state).list_facts(date).await {
        Ok(facts) => {
            (StatusCode::OK, Json(json!({ "count": facts.len(), "facts": facts }))).into_response()
        }
        Err(e) => err(e),
    }
}

async fn jobs_get<S>(State(state): State<S>) -> impl IntoResponse
where
    S: Send + Sync,
    Pool: FromRef<S>,
{
    match svc(&state).recent_jobs(20).await {
        Ok(jobs) => (StatusCode::OK, Json(json!({ "jobs": jobs }))).into_response(),
        Err(e) => err(e),
    }
}

async fn dashboards_get<S>(State(state): State<S>) -> impl IntoResponse
where
    S: Send + Sync,
    Pool: FromRef<S>,
{
    match svc(&state).list_dashboards().await {
        Ok(dashboards) => {
            (StatusCode::OK, Json(json!({ "dashboards": dashboards }))).into_response()
        }
        Err(e) => err(e),
    }
}

async fn dashboards_ensure<S>(State(state): State<S>) -> impl IntoResponse
where
    S: Send + Sync,
    Pool: FromRef<S>,
{
    match svc(&state).ensure_dashboard().await {
        Ok(report) => (StatusCode::OK, Json(report)).into_response(),
        Err(e) => err(e),
    }
}

fn parse_date_opt(s: Option<&str>) -> Result<Option<NaiveDate>, String> {
    match s.map(str::trim).filter(|x| !x.is_empty()) {
        None => Ok(None),
        Some(s) => NaiveDate::parse_from_str(s, "%Y-%m-%d")
            .map(Some)
            .map_err(|e| format!("invalid date {s:?}: {e}")),
    }
}

fn err(e: super::BiError) -> axum::response::Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorBody {
            error: e.to_string(),
        }),
    )
        .into_response()
}

fn bad(e: String) -> axum::response::Response {
    (StatusCode::BAD_REQUEST, Json(ErrorBody { error: e })).into_response()
}

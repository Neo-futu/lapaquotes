use axum::{
    body::Body,
    extract::State,
    http::{Request, Response, StatusCode, Uri, header::CONTENT_TYPE},
    response::{IntoResponse, Response as AxumResponse},
};
use http::HeaderValue;
use leptos::prelude::*;
use std::convert::Infallible;
use tower::ServiceExt;
use tower_http::services::{ServeDir, fs::ServeFileSystemResponseBody};

use crate::err::AppError;
use crate::err::ErrorTemplate;

pub const MAX_AGE_MONTH: HeaderValue = HeaderValue::from_static("public, max-age=2592000");
// const MAX_AGE_YEAR: HeaderValue = HeaderValue::from_static("public, max-age=31536000");

pub async fn file_and_error_handler(
    uri: Uri,
    State(options): State<LeptosOptions>,
    req: Request<Body>,
) -> AxumResponse {
    let res = get_static_file(uri.clone(), &options.site_root).await.unwrap();
    if res.status() == StatusCode::OK {
        res.into_response()
    } else if is_wasm_bg_alias(&uri) {
        // site.js fetches `<output>_bg.wasm` but on Windows cargo-leptos 0.3.11
        // copies `<output>.wasm` to the pkg dir, so the requested name 404s and
        // hydration dies (every interactive filter silently fails). Rewrite.
        let alias_uri = rewrite_wasm_bg_uri(&uri);
        let alias_res = get_static_file(alias_uri, &options.site_root).await.unwrap();
        if alias_res.status() == StatusCode::OK {
            let mut alias_res = alias_res.into_response();
            let headers = alias_res.headers_mut();
            headers.insert(CONTENT_TYPE, "application/wasm".parse().unwrap());
            alias_res
        } else {
            not_found_response(req).await
        }
    } else {
        not_found_response(req).await
    }
}

fn is_wasm_bg_alias(uri: &Uri) -> bool {
    uri.path().ends_with("/site_bg.wasm")
}

fn rewrite_wasm_bg_uri(uri: &Uri) -> Uri {
    let path = uri.path().replace("site_bg.wasm", "site.wasm");
    let full = uri.to_string();
    let query = full.split_once('?').map(|(_, q)| format!("?{q}")).unwrap_or_default();
    let s = format!("{path}{query}");
    Uri::try_from(s.as_str()).unwrap_or_else(|_| uri.clone())
}

pub async fn serve_file(
    uri: &Uri,
    root: &str,
) -> Result<Response<ServeFileSystemResponseBody>, Infallible> {
    let req = Request::builder().uri(uri).body(Body::empty()).unwrap();
    ServeDir::new(root).oneshot(req).await
}

async fn get_static_file(uri: Uri, root: &str) -> Result<Response<Body>, (StatusCode, String)> {
    let req = Request::builder()
        .uri(uri.clone())
        .body(Body::empty())
        .unwrap();
    match ServeDir::new(root).oneshot(req).await {
        Ok(res) => Ok(res.into_response()),
        Err(err) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {err}"),
        )),
    }
}

pub async fn not_found_response(req: Request<Body>) -> AxumResponse {
    tracing::warn!("404 not_found_response for: {:?}", req.uri());
    let mut errors = Errors::default();
    errors.insert_with_default_key(AppError::NotFound);
    let handler = leptos_axum::render_app_to_stream(
        move || view! { <ErrorTemplate outside_errors=errors.clone()></ErrorTemplate> },
    );
    handler(req).await.into_response()
}

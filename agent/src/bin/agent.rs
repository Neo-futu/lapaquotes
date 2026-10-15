//! Standalone agent HTTP service (same routes as site `/agent/*`, mounted at `/`).
//!
//! ```sh
//! AGENT_ADDR=127.0.0.1:4151 cargo run -p agent --features standalone --bin agent
//! ```

use axum::Router;
use config::ConfigError;
use cornucopia::tokio_postgres;
use deadpool_postgres::Runtime;
use serde::Deserialize;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

#[derive(Debug, Deserialize)]
struct Config {
    pg: deadpool_postgres::Config,
}

impl Config {
    fn from_env() -> Result<Self, ConfigError> {
        config::Config::builder()
            .add_source(config::Environment::default().separator("__"))
            .build()
            .unwrap()
            .try_deserialize()
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::Subscriber::builder()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    dotenvy::dotenv().ok();
    let config = Config::from_env().expect("PG__URL config");
    let pool = config
        .pg
        .create_pool(Some(Runtime::Tokio1), tokio_postgres::NoTls)
        .expect("pg pool");

    let addr = std::env::var("AGENT_ADDR").unwrap_or_else(|_| "127.0.0.1:4151".into());

    let app = Router::new()
        .merge(agent::router::<agent::AgentState>())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .with_state(agent::AgentState::new(pool));

    info!("agent standalone listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await.expect("bind");
    axum::serve(listener, app.into_make_service())
        .await
        .expect("serve");
}

//! BI pipeline CLI: catalog → migrate → sync → verify.
//!
//! ```sh
//! cargo run -p agent --example bi_sync --features standalone -- catalog
//! cargo run -p agent --example bi_sync --features standalone -- migrate
//! cargo run -p agent --example bi_sync --features standalone -- sync 2026-10-07
//! cargo run -p agent --example bi_sync --features standalone -- verify
//! cargo run -p agent --example bi_sync --features standalone -- daily
//! cargo run -p agent --example bi_sync --features standalone -- views 2026-10-08
//! ```
//!
//! Env: `FEISHU_BITABLE_ACCESS_TOKEN`, `FEISHU_BITABLE_APP_TOKEN`,
//! optional `FEISHU_BITABLE_API_BASE` (+ `PG__URL`).

use chrono::NaiveDate;
use config::ConfigError;
use cornucopia::tokio_postgres;
use deadpool_postgres::Runtime;
use serde::Deserialize;

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

async fn pool() -> deadpool_postgres::Pool {
    let cfg = Config::from_env().expect("PG__URL");
    cfg.pg
        .create_pool(Some(Runtime::Tokio1), tokio_postgres::NoTls)
        .expect("pool")
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::Subscriber::builder()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();
    let _ = dotenvy::dotenv();

    let mode = std::env::args().nth(1).unwrap_or_else(|| "verify".into());
    let bi = agent::bi::BiService::new(pool().await);

    match mode.as_str() {
        "catalog" => {
            let tables = bi.refresh_catalog().await.expect("catalog");
            println!("{}", serde_json::to_string_pretty(&tables).unwrap());
        }
        "migrate" => {
            let report = bi.migrate_bitable().await.expect("migrate");
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
        }
        "sync" => {
            let date = std::env::args().nth(2).map(|s| {
                NaiveDate::parse_from_str(&s, "%Y-%m-%d").expect("YYYY-MM-DD")
            });
            let report = bi.sync_day(date).await.expect("sync");
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
        }
        "facts" => {
            let date = std::env::args().nth(2).map(|s| {
                NaiveDate::parse_from_str(&s, "%Y-%m-%d").expect("YYYY-MM-DD")
            });
            let facts = bi.list_facts(date).await.expect("facts");
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "count": facts.len(),
                    "facts": facts,
                }))
                .unwrap()
            );
        }
        "jobs" => {
            let jobs = bi.recent_jobs(20).await.expect("jobs");
            println!("{}", serde_json::to_string_pretty(&jobs).unwrap());
        }
        "dashboard" => {
            let report = bi.ensure_dashboard().await.expect("dashboard");
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
        }
        "views" => {
            let date = std::env::args().nth(2).map(|s| {
                NaiveDate::parse_from_str(&s, "%Y-%m-%d").expect("YYYY-MM-DD")
            });
            let client = agent::bi::bitable::BitableClient::connect().expect("bitable");
            let catalog = bi.refresh_catalog().await.expect("catalog");
            let ledger = catalog
                .iter()
                .find(|t| t.role == agent::bi::BitableRole::QuoteLedger)
                .expect("quote_ledger");
            let report = agent::bi::bitable::ensure_ledger_chart_views_for_date(
                &client,
                &ledger.table_id,
                date,
            )
            .await
            .expect("views");
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
        }
        "daily" => {
            // Idempotent daily: refresh catalog, ensure schema once if needed, sync.
            let catalog = bi.refresh_catalog().await.expect("catalog");
            let need_migrate = catalog.iter().any(|t| {
                !t.fields.iter().any(|f| f.name == "同步键")
                    || (t.role == agent::bi::BitableRole::QuoteLedger
                        && !t.fields.iter().any(|f| f.name == "Bid"))
            });
            if need_migrate {
                let m = bi.migrate_bitable().await.expect("migrate");
                println!("migrated: {}", serde_json::to_string(&m).unwrap());
            }
            let d = bi.ensure_dashboard().await.expect("dashboard");
            println!("dashboard: {}", serde_json::to_string(&d).unwrap());
            let report = bi.sync_day(None).await.expect("sync");
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
        }
        _ => {
            let report = bi.verify().await.expect("verify");
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
            if !report.ok {
                std::process::exit(1);
            }
        }
    }
}

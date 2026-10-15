//! Quote tools for the Rig agent — DynamicTool pattern from
//! `examples/agent_with_tools` (same as `tools.rs` posts).

use chrono::NaiveDate;
use cornucopia::deadpool_postgres::Pool;
use rig::message::{EmptyToolName, ToolName};
use rig::tool::{DynamicTool, ToolOutput};
use serde::Deserialize;
use serde_json::json;

use crate::quote_context::{
    bond_by_date, deposit_by_date, institutions, recent_quote_dates, search_bonds,
};

#[derive(Deserialize)]
struct QuoteDateArgs {
    /// Optional ISO date (YYYY-MM-DD); empty / missing = latest available date.
    #[serde(default)]
    date: Option<String>,
    /// Optional institution name substring filter.
    #[serde(default)]
    institution: Option<String>,
}

#[derive(Deserialize)]
struct BondSearchArgs {
    query: String,
}

#[derive(Deserialize)]
struct EmptyArgs {}

fn parse_date(s: Option<String>) -> Option<NaiveDate> {
    let s = s?;
    let t = s.trim();
    if t.is_empty() {
        return None;
    }
    NaiveDate::parse_from_str(t, "%Y-%m-%d").ok()
}

/// Runtime tools the model must use to read OTC quotes from Postgres.
pub fn quote_tools(pool: Pool) -> Result<Vec<DynamicTool>, EmptyToolName> {
    let pool_dates = pool.clone();
    let pool_deposits = pool.clone();
    let pool_bonds = pool.clone();
    let pool_search = pool.clone();
    let pool_institutions = pool;

    Ok(vec![
        DynamicTool::new(
            ToolName::new("list_recent_quote_dates")?,
            "List the most recent quote dates (YYYY-MM-DD, newest first) that have \
             deposit or bond quotes. Use this first when the user does not give a date.",
            json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }),
            move |_args| {
                let pool = pool_dates.clone();
                Box::pin(async move {
                    let _args: EmptyArgs = EmptyArgs {};
                    match recent_quote_dates(&pool).await {
                        Ok(dates) => Ok(ToolOutput::json(
                            serde_json::to_value(dates).unwrap_or_else(|e| json!({ "error": e.to_string() })),
                        )),
                        Err(e) => Ok(ToolOutput::json(json!({ "error": e.to_string() }))),
                    }
                })
            },
        ),
        DynamicTool::new(
            ToolName::new("get_deposit_quotes")?,
            "Get deposit quotes by optional date (YYYY-MM-DD, default latest) and \
             optional institution name substring. Returns term, currency, bid/ask rates.",
            json!({
                "type": "object",
                "properties": {
                    "date": {
                        "type": "string",
                        "description": "Optional quote date YYYY-MM-DD; omit for the latest date"
                    },
                    "institution": {
                        "type": "string",
                        "description": "Optional institution name substring filter"
                    }
                },
                "additionalProperties": false
            }),
            move |args| {
                let pool = pool_deposits.clone();
                Box::pin(async move {
                    let args: QuoteDateArgs = serde_json::from_value(args).map_err(|error| {
                        rig::tool::ToolExecutionError::invalid_args(error.to_string())
                            .with_source(error)
                    })?;
                    let date = parse_date(args.date);
                    let institution = args.institution;
                    match deposit_by_date(&pool, date, institution).await {
                        Ok(quotes) => Ok(ToolOutput::json(
                            serde_json::to_value(quotes).unwrap_or_else(|e| json!({ "error": e.to_string() })),
                        )),
                        Err(e) => Ok(ToolOutput::json(json!({ "error": e.to_string() }))),
                    }
                })
            },
        ),
        DynamicTool::new(
            ToolName::new("get_bond_quotes")?,
            "Get bond quotes by optional date (YYYY-MM-DD, default latest) and \
             optional institution name substring. Returns code, name, maturity, \
             coupon, bid/ask yields and prices.",
            json!({
                "type": "object",
                "properties": {
                    "date": {
                        "type": "string",
                        "description": "Optional quote date YYYY-MM-DD; omit for the latest date"
                    },
                    "institution": {
                        "type": "string",
                        "description": "Optional institution name substring filter"
                    }
                },
                "additionalProperties": false
            }),
            move |args| {
                let pool = pool_bonds.clone();
                Box::pin(async move {
                    let args: QuoteDateArgs = serde_json::from_value(args).map_err(|error| {
                        rig::tool::ToolExecutionError::invalid_args(error.to_string())
                            .with_source(error)
                    })?;
                    let date = parse_date(args.date);
                    let institution = args.institution;
                    match bond_by_date(&pool, date, institution).await {
                        Ok(quotes) => Ok(ToolOutput::json(
                            serde_json::to_value(quotes).unwrap_or_else(|e| json!({ "error": e.to_string() })),
                        )),
                        Err(e) => Ok(ToolOutput::json(json!({ "error": e.to_string() }))),
                    }
                })
            },
        ),
        DynamicTool::new(
            ToolName::new("search_bond_quotes")?,
            "Search bond quotes by keyword against bond code and bond name \
             (case-insensitive substring). Empty query returns the most recent quotes.",
            json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search keyword, e.g. an ISIN or issuer name"
                    }
                },
                "required": ["query"]
            }),
            move |args| {
                let pool = pool_search.clone();
                Box::pin(async move {
                    let args: BondSearchArgs = serde_json::from_value(args).map_err(|error| {
                        rig::tool::ToolExecutionError::invalid_args(error.to_string())
                            .with_source(error)
                    })?;
                    match search_bonds(&pool, &args.query).await {
                        Ok(quotes) => Ok(ToolOutput::json(
                            serde_json::to_value(quotes).unwrap_or_else(|e| json!({ "error": e.to_string() })),
                        )),
                        Err(e) => Ok(ToolOutput::json(json!({ "error": e.to_string() }))),
                    }
                })
            },
        ),
        DynamicTool::new(
            ToolName::new("list_quote_institutions")?,
            "List institutions that send quotes (name, short name, active flag).",
            json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }),
            move |_args| {
                let pool = pool_institutions.clone();
                Box::pin(async move {
                    let _args: EmptyArgs = EmptyArgs {};
                    match institutions(&pool).await {
                        Ok(rows) => Ok(ToolOutput::json(
                            serde_json::to_value(rows).unwrap_or_else(|e| json!({ "error": e.to_string() })),
                        )),
                        Err(e) => Ok(ToolOutput::json(json!({ "error": e.to_string() }))),
                    }
                })
            },
        ),
    ])
}

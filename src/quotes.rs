use chrono::NaiveDate;
use leptos::{either::Either, prelude::*};
use leptos_meta::{Meta, Title};
use serde::{Deserialize, Serialize};

use crate::hk_calendar::HkHolidayCalendar;
use crate::util::{AlertDanger, Loading};

// ---------------------------------------------------------------------------
// DTOs (NUMERIC -> f64 for serialization; DATE -> NaiveDate serializes as YYYY-MM-DD)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstitutionOption {
    pub id: String,
    pub name: String,
    pub short_name: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DepositQuoteView {
    pub id: String,
    pub quote_date: NaiveDate,
    pub institution_id: String,
    pub institution_name: String,
    pub institution_short_name: Option<String>,
    pub term: String,
    pub currency: String,
    pub bid: Option<f64>,
    pub ask: Option<f64>,
    pub note: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BondQuoteView {
    pub id: String,
    pub quote_date: NaiveDate,
    pub institution_id: String,
    pub institution_name: String,
    pub institution_short_name: Option<String>,
    pub bond_code: String,
    pub bond_name: Option<String>,
    pub maturity_date: Option<NaiveDate>,
    pub coupon: Option<f64>,
    pub bid_yield: Option<f64>,
    pub ask_yield: Option<f64>,
    pub bid_price: Option<f64>,
    pub ask_price: Option<f64>,
    pub direction: Option<String>,
    pub note: Option<String>,
}

// ---------------------------------------------------------------------------
// Calendar date picker (custom popover, no JS deps)
// ---------------------------------------------------------------------------

// Date filter uses the browser's native date input: clicking the calendar icon
// on the right opens the platform date picker (no JS crate needed).
// Empty value = latest available quote date.

// ---------------------------------------------------------------------------
// Page
// ---------------------------------------------------------------------------

#[component]
pub fn QuotesPage() -> impl IntoView {
    // tab: 0 = deposits, 1 = bonds
    let tab = RwSignal::new(0u8);
    let date = RwSignal::new(String::new());
    let institution = RwSignal::new(String::new());

    let latest = Resource::new_blocking(|| (), move |_| get_latest_quote_date());
    let inst_loader = Resource::new_blocking(|| (), move |_| get_quote_institutions());

    // Default DATE to the latest quote day once known.
    Effect::new(move |_| {
        if date.get_untracked().is_empty() {
            if let Some(Ok(Some(latest))) = latest.get() {
                date.set(latest.format("%Y-%m-%d").to_string());
            }
        }
    });

    // DATE drives the server fetch (blocking so SSR renders real rows, not Loading).
    // INSTITUTION is applied after await so changing the select re-renders instantly.
    let date_key = move || parse_date(&date.get());
    let deposits = Resource::new_blocking(date_key, move |date_opt| async move {
        get_deposit_quotes(date_opt, None).await
    });
    let bonds = Resource::new_blocking(date_key, move |date_opt| async move {
        get_bond_quotes(date_opt, None).await
    });

    let refresh = move |_| {
        deposits.refetch();
        bonds.refetch();
        inst_loader.refetch();
        latest.refetch();
    };

    view! {
        <Title text="Quotes" />
        <Meta name="description" content="Daily OTC deposit and bond quotes from institutions." />
        <section class="Quotes">
            <header class="Quotes-header">
                <h1>
                    <span class="Quotes-h1-accent">OTC</span>
                    " QUOTES"
                </h1>
                <p class="Quotes-lead">
                    "Daily deposit & bond indications from contributing institutions."
                </p>
            </header>

            <div class="Quotes-body">
                <div class="Quotes-main">
                    <nav class="Quotes-tabs" role="tablist">
                        <button
                            type="button"
                            role="tab"
                            class=move || tab_cls(tab.get() == 0)
                            on:click=move |_| tab.set(0)
                        >
                            "DEPOSITS"
                        </button>
                        <button
                            type="button"
                            role="tab"
                            class=move || tab_cls(tab.get() == 1)
                            on:click=move |_| tab.set(1)
                        >
                            "BONDS"
                        </button>
                    </nav>

                    <div class="Quotes-filters">
                        <label class="Quotes-field">
                            <span>DATE</span>
                            <input
                                type="date"
                                prop:value=move || date.get()
                                on:input=move |ev| date.set(event_target_value(&ev))
                                on:change=move |ev| date.set(event_target_value(&ev))
                            />
                        </label>
                        <label class="Quotes-field">
                            <span>INSTITUTION</span>
                            <Suspense fallback=|| {
                                view! {
                                    <select disabled>
                                        <option>"Loading…"</option>
                                    </select>
                                }
                            }>
                                {move || Suspend::new(async move {
                                    let items = inst_loader.await.unwrap_or_default();
                                    view! {
                                        <select
                                            prop:value=move || institution.get()
                                            on:input=move |ev| institution.set(event_target_value(&ev))
                                            on:change=move |ev| institution.set(event_target_value(&ev))
                                        >
                                            <option value="">"All institutions"</option>
                                            {items
                                                .into_iter()
                                                .map(|inst| {
                                                    let label = inst
                                                        .short_name
                                                        .clone()
                                                        .unwrap_or_else(|| inst.name.clone());
                                                    let id = inst.id.clone();
                                                    view! { <option value=id>{label}</option> }
                                                })
                                                .collect_view()}
                                        </select>
                                    }
                                })}
                            </Suspense>
                        </label>
                        <div class="Quotes-field Quotes-field--action">
                            <span>" "</span>
                            <button type="button" class="Quotes-refresh" on:click=refresh>
                                "REFRESH"
                            </button>
                        </div>
                    </div>

                    <Show
                        when=move || tab.get() == 0
                        fallback=move || {
                            view! {
                                <Suspense fallback=move || view! { <Loading /> }>
                                    {move || {
                                        let inst = institution.get();
                                        let filter_date = parse_date(&date.get());
                                        Suspend::new(async move {
                                            match bonds.await {
                                                Err(e) => {
                                                    Either::Left(view! { <AlertDanger text=e.to_string() /> })
                                                }
                                                Ok(quotes) => {
                                                    let quotes = filter_bonds(quotes, &inst);
                                                    Either::Right(view! {
                                                        <BondQuotesPanel quotes filter_date />
                                                    })
                                                }
                                            }
                                        })
                                    }}
                                </Suspense>
                            }
                            .into_any()
                        }
                    >
                        <Suspense fallback=move || view! { <Loading /> }>
                            {move || {
                                let inst = institution.get();
                                let filter_date = parse_date(&date.get());
                                Suspend::new(async move {
                                    match deposits.await {
                                        Err(e) => {
                                            Either::Left(view! { <AlertDanger text=e.to_string() /> })
                                        }
                                        Ok(quotes) => {
                                            let quotes = filter_deposits(quotes, &inst);
                                            Either::Right(view! {
                                                <DepositQuotesPanel quotes filter_date />
                                            })
                                        }
                                    }
                                })
                            }}
                        </Suspense>
                    </Show>
                </div>

                <aside class="Quotes-aside">
                    <HkHolidayCalendar quote_date=date />
                </aside>
            </div>
        </section>
    }
}

fn filter_deposits(quotes: Vec<DepositQuoteView>, institution_id: &str) -> Vec<DepositQuoteView> {
    let id = institution_id.trim();
    if id.is_empty() {
        return quotes;
    }
    quotes
        .into_iter()
        .filter(|q| q.institution_id == id)
        .collect()
}

fn filter_bonds(quotes: Vec<BondQuoteView>, institution_id: &str) -> Vec<BondQuoteView> {
    let id = institution_id.trim();
    if id.is_empty() {
        return quotes;
    }
    quotes
        .into_iter()
        .filter(|q| q.institution_id == id)
        .collect()
}

// ---------------------------------------------------------------------------
// Deposits panel — grouped by term with best bid/ask & tightest spread highlighted
// ---------------------------------------------------------------------------

#[component]
fn DepositQuotesPanel(
    quotes: Vec<DepositQuoteView>,
    filter_date: Option<NaiveDate>,
) -> impl IntoView {
    if quotes.is_empty() {
        let label = filter_date
            .map(|d| d.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| "selected filters".into());
        Either::Left(view! {
            <p class="Quotes-empty">
                "No deposit quotes for "
                {label}
                "."
            </p>
        })
    } else {
        Either::Right(view! {
            <DepositStatsStrip quotes=quotes.clone() filter_date />
            <DepositBestPanel quotes=quotes.clone() />
            <div class="Quotes-table-wrap">
                <table class="Quotes-table">
                    <thead>
                        <tr>
                            <th>Term</th>
                            <th>Institution</th>
                            <th class="num">Bid %</th>
                            <th class="num">Ask %</th>
                            <th class="num">Mid</th>
                            <th class="num">Sprd (bp)</th>
                            <th>Note</th>
                        </tr>
                    </thead>
                    <tbody>
                        {quotes
                            .clone()
                            .into_iter()
                            .map(|q| {
                                let term = q.term.clone();
                                let short = q.institution_short_name.clone();
                                let short_sub = short.clone();
                                let best = best_deposit(&quotes, &q);
                                let mid = deposit_mid(&q);
                                let sprd = deposit_spread_bp(&q);
                                view! {
                                    <tr>
                                        <td class="Quotes-term">{term}</td>
                                        <td>
                                            {q.institution_name.clone()}
                                            <Show when=move || short.is_some()>
                                                <span class="Quotes-sub">{short_sub.clone().unwrap_or_default()}</span>
                                            </Show>
                                        </td>
                                        <td class={bid_cell_cls(best.bid)}>{fmt_opt(q.bid)}</td>
                                        <td class={ask_cell_cls(best.ask)}>{fmt_opt(q.ask)}</td>
                                        <td class="num Quotes-mid">{mid}</td>
                                        <td class={sprd_cell_cls(best.tightest)}>{sprd}</td>
                                        <td class="Quotes-note">{q.note.clone().unwrap_or_default()}</td>
                                    </tr>
                                }
                            })
                            .collect_view()}
                    </tbody>
                </table>
            </div>
            <p class="Quotes-legend">
                <span class="Quotes-tag Quotes-tag--best">BEST</span>
                " — highest bid / lowest ask per term; tightest spread per term. "
                "All rates are indications only."
            </p>
        })
    }
}

/// Per-term best bid / best ask / tightest spread flags for one deposit row.
struct DepositBest {
    bid: bool,
    ask: bool,
    tightest: bool,
}

fn best_deposit(all: &[DepositQuoteView], row: &DepositQuoteView) -> DepositBest {
    let same_term = |r: &DepositQuoteView| r.term == row.term;
    let max_bid = all
        .iter()
        .filter(|r| same_term(r))
        .filter_map(|r| r.bid)
        .fold(f64::NAN, f64::max);
    let min_ask = all
        .iter()
        .filter(|r| same_term(r))
        .filter_map(|r| r.ask)
        .fold(f64::INFINITY, f64::min);
    let min_spread = all
        .iter()
        .filter(|r| same_term(r))
        .filter_map(|r| match (r.bid, r.ask) {
            (Some(b), Some(a)) if a >= b => Some(a - b),
            _ => None,
        })
        .fold(f64::INFINITY, f64::min);
    DepositBest {
        bid: row.bid.is_some_and(|b| b == max_bid),
        ask: row.ask.is_some_and(|a| a == min_ask),
        tightest: match (row.bid, row.ask) {
            (Some(b), Some(a)) if a >= b => (a - b) == min_spread,
            _ => false,
        },
    }
}

fn deposit_mid(q: &DepositQuoteView) -> String {
    match (q.bid, q.ask) {
        (Some(b), Some(a)) => format!("{:.3}", (b + a) / 2.0),
        _ => "—".into(),
    }
}

fn deposit_spread_bp(q: &DepositQuoteView) -> String {
    match (q.bid, q.ask) {
        (Some(b), Some(a)) if a >= b => format!("{:.0}", (a - b) * 100.0),
        _ => "—".into(),
    }
}

#[component]
fn DepositStatsStrip(
    quotes: Vec<DepositQuoteView>,
    filter_date: Option<NaiveDate>,
) -> impl IntoView {
    // Prefer the shared filter / calendar date so the DATE row stays in sync.
    let date = filter_date.or_else(|| quotes.first().map(|q| q.quote_date));
    let institutions: usize = quotes
        .iter()
        .map(|q| q.institution_id.as_str())
        .collect::<std::collections::HashSet<_>>()
        .len();
    let terms: usize = quotes
        .iter()
        .map(|q| q.term.as_str())
        .collect::<std::collections::HashSet<_>>()
        .len();
    // Tightest spread across the whole day.
    let tightest = quotes
        .iter()
        .filter_map(|q| match (q.bid, q.ask) {
            (Some(b), Some(a)) if a >= b => Some((q.term.clone(), a - b)),
            _ => None,
        })
        .min_by(|a, b| a.1.total_cmp(&b.1));
    let stats = [
        (
            "DATE",
            date.map(|d| d.format("%Y-%m-%d").to_string())
                .unwrap_or_else(|| "—".into()),
        ),
        ("INSTITUTIONS", institutions.to_string()),
        ("TERMS", terms.to_string()),
        ("LINES", quotes.len().to_string()),
        (
            "TIGHTEST SPRD",
            tightest
                .map(|(t, s)| format!("{} {:.0}bp", t, s * 100.0))
                .unwrap_or_else(|| "—".into()),
        ),
    ];
    view! {
        <div class="Quotes-stats">
            {stats
                .into_iter()
                .map(|(label, value)| {
                    view! {
                        <div class="Quotes-stat">
                            <span class="Quotes-stat-label">{label}</span>
                            <span class="Quotes-stat-value">{value}</span>
                        </div>
                    }
                })
                .collect_view()}
        </div>
    }
}

/// Aggregated best bid / best ask / tightest spread for one term group.
#[derive(Clone, Debug)]
struct DepositBestRow {
    term: String,
    best_bid: Option<(String, Option<String>, f64)>, // (institution, short_name, rate)
    best_ask: Option<(String, Option<String>, f64)>,
    tightest: Option<(String, Option<String>, f64)>, // (institution, short_name, spread)
    mid: Option<f64>,
}

fn aggregate_deposit_best(quotes: &[DepositQuoteView]) -> Vec<DepositBestRow> {
    use std::collections::HashMap;
    // Preserve first-seen order (rows already arrive sorted by term_sort in SQL).
    let mut order: Vec<String> = Vec::new();
    let mut groups: HashMap<String, Vec<&DepositQuoteView>> = HashMap::new();
    for q in quotes {
        let term = q.term.clone();
        if !groups.contains_key(&term) {
            order.push(term.clone());
        }
        groups.entry(term).or_default().push(q);
    }
    order
        .into_iter()
        .map(|term| {
            let rows = groups.remove(&term).unwrap_or_default();
            let best_bid = rows
                .iter()
                .filter_map(|r| r.bid.map(|b| (r.institution_name.clone(), r.institution_short_name.clone(), b)))
                .max_by(|a, b| a.2.total_cmp(&b.2));
            let best_ask = rows
                .iter()
                .filter_map(|r| r.ask.map(|a| (r.institution_name.clone(), r.institution_short_name.clone(), a)))
                .min_by(|a, b| a.2.total_cmp(&b.2));
            let tightest = rows
                .iter()
                .filter_map(|r| match (r.bid, r.ask) {
                    (Some(b), Some(a)) if a >= b => {
                        Some((r.institution_name.clone(), r.institution_short_name.clone(), a - b))
                    }
                    _ => None,
                })
                .min_by(|a, b| a.2.total_cmp(&b.2));
            let mid = match (&best_bid, &best_ask) {
                (Some(b), Some(a)) => Some((b.2 + a.2) / 2.0),
                _ => None,
            };
            DepositBestRow {
                term,
                best_bid,
                best_ask,
                tightest,
                mid,
            }
        })
        .collect()
}

#[component]
fn DepositBestPanel(quotes: Vec<DepositQuoteView>) -> impl IntoView {
    let rows = aggregate_deposit_best(&quotes);
    view! {
        <div class="Quotes-best">
            <div class="Quotes-best-title">
                <span class="Quotes-tag Quotes-tag--best">BEST</span>
                " DEPOSITS"
            </div>
            <div class="Quotes-table-wrap">
                <table class="Quotes-table Quotes-table--best">
                    <thead>
                        <tr class="Quotes-thead-group">
                            <th rowspan="2">Term</th>
                            <th colspan="2" class="Quotes-grp Quotes-grp--bid">Best Bid</th>
                            <th colspan="2" class="Quotes-grp Quotes-grp--ask">Best Ask</th>
                            <th colspan="2" class="Quotes-grp Quotes-grp--sprd">Tightest Spread</th>
                            <th rowspan="2" class="num">Mid</th>
                        </tr>
                        <tr>
                            <th class="num">Rate</th>
                            <th>By</th>
                            <th class="num">Rate</th>
                            <th>By</th>
                            <th class="num">Sprd</th>
                            <th>By</th>
                        </tr>
                    </thead>
                    <tbody>
                        {rows
                            .into_iter()
                            .map(|r| {
                                let (bid_v, bid_inst) = opt_rate_inst(&r.best_bid);
                                let (ask_v, ask_inst) = opt_rate_inst(&r.best_ask);
                                let (sp_v, sp_inst) = opt_rate_inst_bp(&r.tightest);
                                view! {
                                    <tr>
                                        <td class="Quotes-term">{r.term}</td>
                                        <td class="num Quotes-best-bid">{bid_v}</td>
                                        <td class="Quotes-bester">{bid_inst}</td>
                                        <td class="num Quotes-best-ask">{ask_v}</td>
                                        <td class="Quotes-bester">{ask_inst}</td>
                                        <td class="num Quotes-best-sprd">{sp_v}</td>
                                        <td class="Quotes-bester">{sp_inst}</td>
                                        <td class="num Quotes-mid">{r.mid.map(|m| format!("{m:.3}")).unwrap_or_else(|| "—".into())}</td>
                                    </tr>
                                }
                            })
                            .collect_view()}
                    </tbody>
                </table>
            </div>
        </div>
    }
}

fn opt_rate_inst(v: &Option<(String, Option<String>, f64)>) -> (String, String) {
    match v {
        Some((name, short, rate)) => {
            let inst = short.clone().unwrap_or_else(|| name.clone());
            (format!("{rate:.4}"), inst)
        }
        None => ("—".into(), "—".into()),
    }
}

/// Like [`opt_rate_inst`] but formats the value as basis points (rates are %).
fn opt_rate_inst_bp(v: &Option<(String, Option<String>, f64)>) -> (String, String) {
    match v {
        Some((name, short, spread)) => {
            let inst = short.clone().unwrap_or_else(|| name.clone());
            (format!("{:.0} bp", spread * 100.0), inst)
        }
        None => ("—".into(), "—".into()),
    }
}

// ---------------------------------------------------------------------------
// Bonds panel — per bond code best bid yield (lowest) / best ask yield (highest)
// ---------------------------------------------------------------------------

#[component]
fn BondQuotesPanel(
    quotes: Vec<BondQuoteView>,
    filter_date: Option<NaiveDate>,
) -> impl IntoView {
    if quotes.is_empty() {
        let label = filter_date
            .map(|d| d.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| "selected filters".into());
        Either::Left(view! {
            <p class="Quotes-empty">
                "No bond quotes for "
                {label}
                "."
            </p>
        })
    } else {
        Either::Right(view! {
            <BondBestPanel quotes=quotes.clone() />
            <div class="Quotes-table-wrap">
                <table class="Quotes-table">
                    <thead>
                        <tr>
                            <th>Code</th>
                            <th>Name</th>
                            <th class="num">Maturity</th>
                            <th class="num">Cpn %</th>
                            <th>Institution</th>
                            <th class="num">Bid Yld</th>
                            <th class="num">Ask Yld</th>
                            <th class="num">Sprd (bp)</th>
                            <th class="num">Bid Px</th>
                            <th class="num">Ask Px</th>
                            <th>Dir</th>
                            <th>Note</th>
                        </tr>
                    </thead>
                    <tbody>
                        {quotes
                            .clone()
                            .into_iter()
                            .map(|q| {
                                let code = q.bond_code.clone();
                                let short = q.institution_short_name.clone();
                                let short_sub = short.clone();
                                let best = best_bond(&quotes, &q);
                                view! {
                                    <tr>
                                        <td class="Quotes-term">{code}</td>
                                        <td>{q.bond_name.clone().unwrap_or_default()}</td>
                                        <td class="num">{fmt_date(q.maturity_date)}</td>
                                        <td class="num">{fmt_opt(q.coupon)}</td>
                                        <td>
                                            {q.institution_name.clone()}
                                            <Show when=move || short.is_some()>
                                                <span class="Quotes-sub">{short_sub.clone().unwrap_or_default()}</span>
                                            </Show>
                                        </td>
                                        <td class={bond_bid_cell_cls(best.bid)}>{fmt_opt(q.bid_yield)}</td>
                                        <td class={bond_ask_cell_cls(best.ask)}>{fmt_opt(q.ask_yield)}</td>
                                        <td class="num Quotes-sprd">{bond_spread_bp(&q)}</td>
                                        <td class="num Quotes-bid">{fmt_opt(q.bid_price)}</td>
                                        <td class="num Quotes-ask">{fmt_opt(q.ask_price)}</td>
                                        <td>{q.direction.clone().unwrap_or_default()}</td>
                                        <td>{q.note.clone().unwrap_or_default()}</td>
                                    </tr>
                                }
                            })
                            .collect_view()}
                    </tbody>
                </table>
            </div>
            <p class="Quotes-legend">
                <span class="Quotes-tag Quotes-tag--best">BEST</span>
                " — for yields: lowest bid / highest ask per bond (from the client's perspective). "
                "All quotes are indications only."
            </p>
        })
    }
}

/// Per-bond best bid yield (lowest = best for buyer) / best ask yield (highest).
struct BondBest {
    bid: bool,
    ask: bool,
}

fn best_bond(all: &[BondQuoteView], row: &BondQuoteView) -> BondBest {
    let same_bond = |r: &BondQuoteView| r.bond_code == row.bond_code;
    let min_bid_y = all
        .iter()
        .filter(|r| same_bond(r))
        .filter_map(|r| r.bid_yield)
        .fold(f64::INFINITY, f64::min);
    let max_ask_y = all
        .iter()
        .filter(|r| same_bond(r))
        .filter_map(|r| r.ask_yield)
        .fold(f64::NAN, f64::max);
    BondBest {
        bid: row.bid_yield.is_some_and(|b| b == min_bid_y),
        ask: row.ask_yield.is_some_and(|a| a == max_ask_y),
    }
}

fn bond_spread_bp(q: &BondQuoteView) -> String {
    match (q.bid_yield, q.ask_yield) {
        (Some(b), Some(a)) => format!("{:.0}", (a - b).abs() * 100.0),
        _ => "—".into(),
    }
}

/// Aggregated best (client-perspective) yields per bond code.
#[derive(Clone, Debug)]
struct BondBestRow {
    code: String,
    name: Option<String>,
    best_bid_y: Option<(String, Option<String>, f64)>,  // lowest bid yield
    best_ask_y: Option<(String, Option<String>, f64)>,  // highest ask yield
    tightest: Option<(String, Option<String>, f64)>,    // smallest |ask-bid|
    institutions: usize,
}

fn aggregate_bond_best(quotes: &[BondQuoteView]) -> Vec<BondBestRow> {
    use std::collections::HashMap;
    let mut order: Vec<String> = Vec::new();
    let mut groups: HashMap<String, Vec<&BondQuoteView>> = HashMap::new();
    for q in quotes {
        let code = q.bond_code.clone();
        if !groups.contains_key(&code) {
            order.push(code.clone());
        }
        groups.entry(code).or_default().push(q);
    }
    order
        .into_iter()
        .map(|code| {
            let rows = groups.remove(&code).unwrap_or_default();
            let name = rows.iter().find_map(|r| r.bond_name.clone());
            let institutions = rows
                .iter()
                .map(|r| r.institution_id.as_str())
                .collect::<std::collections::HashSet<_>>()
                .len();
            let best_bid_y = rows
                .iter()
                .filter_map(|r| {
                    r.bid_yield
                        .map(|b| (r.institution_name.clone(), r.institution_short_name.clone(), b))
                })
                .min_by(|a, b| a.2.total_cmp(&b.2));
            let best_ask_y = rows
                .iter()
                .filter_map(|r| {
                    r.ask_yield
                        .map(|a| (r.institution_name.clone(), r.institution_short_name.clone(), a))
                })
                .max_by(|a, b| a.2.total_cmp(&b.2));
            let tightest = rows
                .iter()
                .filter_map(|r| match (r.bid_yield, r.ask_yield) {
                    (Some(b), Some(a)) => Some((
                        r.institution_name.clone(),
                        r.institution_short_name.clone(),
                        (a - b).abs(),
                    )),
                    _ => None,
                })
                .min_by(|a, b| a.2.total_cmp(&b.2));
            BondBestRow {
                code,
                name,
                best_bid_y,
                best_ask_y,
                tightest,
                institutions,
            }
        })
        .collect()
}

#[component]
fn BondBestPanel(quotes: Vec<BondQuoteView>) -> impl IntoView {
    let rows = aggregate_bond_best(&quotes);
    view! {
        <div class="Quotes-best">
            <div class="Quotes-best-title">
                <span class="Quotes-tag Quotes-tag--best">BEST</span>
                " BONDS"
            </div>
            <div class="Quotes-table-wrap">
                <table class="Quotes-table Quotes-table--best">
                    <thead>
                        <tr class="Quotes-thead-group">
                            <th rowspan="2">Code</th>
                            <th rowspan="2">Name</th>
                            <th colspan="2" class="Quotes-grp Quotes-grp--bid">Best Bid Yld</th>
                            <th colspan="2" class="Quotes-grp Quotes-grp--ask">Best Ask Yld</th>
                            <th colspan="2" class="Quotes-grp Quotes-grp--sprd">Tightest Spread</th>
                            <th rowspan="2" class="num">Quotes</th>
                        </tr>
                        <tr>
                            <th class="num">Yld</th>
                            <th>By</th>
                            <th class="num">Yld</th>
                            <th>By</th>
                            <th class="num">Sprd</th>
                            <th>By</th>
                        </tr>
                    </thead>
                    <tbody>
                        {rows
                            .into_iter()
                            .map(|r| {
                                let (bid_v, bid_inst) = opt_rate_inst(&r.best_bid_y);
                                let (ask_v, ask_inst) = opt_rate_inst(&r.best_ask_y);
                                let (sp_v, _sp_inst) = opt_rate_inst_bp(&r.tightest);
                                view! {
                                    <tr>
                                        <td class="Quotes-term">{r.code}</td>
                                        <td>{r.name.unwrap_or_default()}</td>
                                        <td class="num Quotes-best-bid">{bid_v}</td>
                                        <td class="Quotes-bester">{bid_inst}</td>
                                        <td class="num Quotes-best-ask">{ask_v}</td>
                                        <td class="Quotes-bester">{ask_inst}</td>
                                        <td class="num Quotes-best-sprd">{sp_v}</td>
                                        <td class="num">{r.institutions.to_string()}</td>
                                    </tr>
                                }
                            })
                            .collect_view()}
                    </tbody>
                </table>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

fn tab_cls(active: bool) -> &'static str {
    if active {
        "Quotes-tab Quotes-tab--active"
    } else {
        "Quotes-tab"
    }
}

fn parse_date(s: &str) -> Option<NaiveDate> {
    let t = s.trim();
    if t.is_empty() {
        return None;
    }
    NaiveDate::parse_from_str(t, "%Y-%m-%d").ok()
}

fn fmt_opt(v: Option<f64>) -> String {
    v.map(|x| format!("{x:.4}")).unwrap_or_else(|| "—".into())
}

fn fmt_date(d: Option<NaiveDate>) -> String {
    d.map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "—".into())
}

fn dec_to_f64(d: Option<rust_decimal::Decimal>) -> Option<f64> {
    d.and_then(|d| d.try_into().ok())
}

// Rate colors always match bid(green) / ask(red) / sprd(amber); BEST rows get stronger bg.
fn bid_cell_cls(best: bool) -> &'static str {
    if best {
        "num Quotes-bid Quotes-best-bid"
    } else {
        "num Quotes-bid"
    }
}
fn ask_cell_cls(best: bool) -> &'static str {
    if best {
        "num Quotes-ask Quotes-best-ask"
    } else {
        "num Quotes-ask"
    }
}
fn sprd_cell_cls(tightest: bool) -> &'static str {
    if tightest {
        "num Quotes-sprd Quotes-best-sprd"
    } else {
        "num Quotes-sprd"
    }
}
fn bond_bid_cell_cls(best: bool) -> &'static str {
    if best {
        "num Quotes-bid Quotes-best-bid"
    } else {
        "num Quotes-bid"
    }
}
fn bond_ask_cell_cls(best: bool) -> &'static str {
    if best {
        "num Quotes-ask Quotes-best-ask"
    } else {
        "num Quotes-ask"
    }
}

// ---------------------------------------------------------------------------
// server functions
// ---------------------------------------------------------------------------

#[server(GetQuoteInstitutions, "/api")]
pub async fn get_quote_institutions() -> Result<Vec<InstitutionOption>, ServerFnError> {
    let db = crate::server::db::use_db().await?;
    let rows = cornucopia::queries::quote::institutions()
        .bind(&db)
        .all()
        .await
        .map_err(|e| lib::emsg(e, "Quote institutions list"))?;
    Ok(rows
        .into_iter()
        .map(|r| InstitutionOption {
            id: r.id,
            name: r.name,
            short_name: r.short_name,
        })
        .collect())
}

#[server(GetLatestQuoteDate, "/api")]
pub async fn get_latest_quote_date() -> Result<Option<NaiveDate>, ServerFnError> {
    let db = crate::server::db::use_db().await?;
    let d = cornucopia::queries::quote::latest_deposit_date()
        .bind(&db)
        .opt()
        .await
        .map_err(|e| lib::emsg(e, "Latest deposit date"))?;
    let b = cornucopia::queries::quote::latest_bond_date()
        .bind(&db)
        .opt()
        .await
        .map_err(|e| lib::emsg(e, "Latest bond date"))?;
    Ok(d.flatten().max(b.flatten()))
}

#[server(GetDepositQuotes, "/api")]
pub async fn get_deposit_quotes(
    quote_date: Option<NaiveDate>,
    institution_id: Option<String>,
) -> Result<Vec<DepositQuoteView>, ServerFnError> {
    let db = crate::server::db::use_db().await?;
    let institution_ref: Option<String> = institution_id;
    let rows = cornucopia::queries::quote::deposits_by_date()
        .bind(&db, &quote_date, &institution_ref)
        .all()
        .await
        .map_err(|e| lib::emsg(e, "Deposit quotes list"))?;
    Ok(rows
        .into_iter()
        .map(|r| DepositQuoteView {
            id: r.id,
            quote_date: r.quote_date,
            institution_id: r.institution_id,
            institution_name: r.institution_name,
            institution_short_name: r.institution_short_name,
            term: r.term,
            currency: r.currency,
            bid: dec_to_f64(r.bid_rate),
            ask: dec_to_f64(r.ask_rate),
            note: r.note,
        })
        .collect())
}

#[server(GetBondQuotes, "/api")]
pub async fn get_bond_quotes(
    quote_date: Option<NaiveDate>,
    institution_id: Option<String>,
) -> Result<Vec<BondQuoteView>, ServerFnError> {
    let db = crate::server::db::use_db().await?;
    let institution_ref: Option<String> = institution_id;
    let rows = cornucopia::queries::quote::bonds_by_date()
        .bind(&db, &quote_date, &institution_ref)
        .all()
        .await
        .map_err(|e| lib::emsg(e, "Bond quotes list"))?;
    Ok(rows
        .into_iter()
        .map(|r| BondQuoteView {
            id: r.id,
            quote_date: r.quote_date,
            institution_id: r.institution_id,
            institution_name: r.institution_name,
            institution_short_name: r.institution_short_name,
            bond_code: r.bond_code,
            bond_name: r.bond_name,
            maturity_date: r.maturity_date,
            coupon: dec_to_f64(r.coupon_rate),
            bid_yield: dec_to_f64(r.bid_yield),
            ask_yield: dec_to_f64(r.ask_yield),
            bid_price: dec_to_f64(r.bid_price),
            ask_price: dec_to_f64(r.ask_price),
            direction: r.direction,
            note: r.note,
        })
        .collect())
}

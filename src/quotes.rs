use chrono::NaiveDate;
use leptos::{either::Either, prelude::*};
use leptos_meta::{Meta, Title};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::hk_calendar::{today_local, HkHolidayCalendar};
use crate::util::{AlertDanger, Loading};

// ---------------------------------------------------------------------------
// DTOs (NUMERIC -> f64 for serialization; DATE -> NaiveDate serializes as YYYY-MM-DD)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstitutionOption {
    pub id: String,
    pub name: String,
    pub short_name: Option<String>,
    pub cn_name: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DepositQuoteView {
    pub id: String,
    pub quote_date: NaiveDate,
    pub institution_id: String,
    pub institution_name: String,
    pub institution_short_name: Option<String>,
    pub institution_cn_name: Option<String>,
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
    pub institution_cn_name: Option<String>,
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
// Default = today (Asia/Hong_Kong calendar day on SSR; browser local on hydrate).

// ---------------------------------------------------------------------------
// Page
// ---------------------------------------------------------------------------

#[component]
pub fn QuotesPage() -> impl IntoView {
    // tab: 0 = deposits, 1 = bonds
    let tab = RwSignal::new(0u8);
    let date = RwSignal::new(today_local().format("%Y-%m-%d").to_string());
    let institution = RwSignal::new(String::new());

    let inst_loader = Resource::new_blocking(|| (), move |_| get_quote_institutions());

    // DATE drives the server fetch (blocking so SSR renders real rows, not Loading).
    // INSTITUTION is applied after await so changing the select re-renders instantly.
    let date_key = move || parse_date(&date.get());
    let deposits = Resource::new_blocking(date_key, move |date_opt| async move {
        get_deposit_quotes(date_opt, None).await
    });
    let bonds = Resource::new_blocking(date_key, move |date_opt| async move {
        get_bond_quotes(date_opt, None).await
    });

    let check_tick = RwSignal::new(0u32);
    let refresh = move |_| {
        deposits.refetch();
        bonds.refetch();
        inst_loader.refetch();
        check_tick.update(|n| *n = n.wrapping_add(1));
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
                                                    let label = institution_filter_label(&inst);
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
                    <IngestCheckPanel quote_date=date check_tick />
                </aside>
            </div>
        </section>
    }
}

// ---------------------------------------------------------------------------
// Ingest day-check (below calendar): per-institution mail ingest status
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IngestDayCheckView {
    pub institution_key: String,
    pub short_name: String,
    pub cn_name: Option<String>,
    pub status: String,
    pub rows_n: i32,
    pub mail_count: i32,
    pub last_message: Option<String>,
    /// Outlook mail ReceivedTime as `HH:MM` (local), not relative ago.
    pub received_hm: Option<String>,
}

#[component]
fn IngestCheckPanel(
    quote_date: RwSignal<String>,
    check_tick: RwSignal<u32>,
) -> impl IntoView {
    let checks = Resource::new(
        move || (parse_date(&quote_date.get()), check_tick.get()),
        move |(d, _tick)| async move {
            match d {
                Some(day) => get_ingest_day_checks(day).await,
                None => Ok(Vec::new()),
            }
        },
    );

    view! {
        <div class="IngestCheck" aria-label="Institution ingest checks">
            <div class="IngestCheck-head">
                <span class="IngestCheck-title">"Ingest Check"</span>
                <span class="IngestCheck-sub">
                    {move || {
                        parse_date(&quote_date.get())
                            .map(|d| d.format("%Y-%m-%d").to_string())
                            .unwrap_or_else(|| "—".into())
                    }}
                </span>
            </div>
            <Suspense fallback=move || {
                view! { <p class="IngestCheck-loading">"…"</p> }
            }>
                {move || Suspend::new(async move {
                    match checks.await {
                        Err(e) => Either::Left(view! {
                            <p class="IngestCheck-err">{e.to_string()}</p>
                        }),
                        Ok(rows) if rows.is_empty() => Either::Right(Either::Left(view! {
                            <p class="IngestCheck-empty">"No institution roster for this date."</p>
                        })),
                        Ok(rows) => {
                            let ok_n = rows.iter().filter(|r| r.status == "ok").count();
                            let total = rows.len();
                            Either::Right(Either::Right(view! {
                                <div class="IngestCheck-summary">
                                    <span class=move || {
                                        if ok_n == total && total > 0 {
                                            "IngestCheck-summary-ok"
                                        } else {
                                            "IngestCheck-summary-mix"
                                        }
                                    }>
                                        {format!("{ok_n}/{total} OK")}
                                    </span>
                                </div>
                                <ul class="IngestCheck-list">
                                    {rows
                                        .into_iter()
                                        .map(|r| {
                                            let mark = match r.status.as_str() {
                                                "ok" => "✓",
                                                "error" => "✗",
                                                "skip" => "△",
                                                _ => "○",
                                            };
                                            let label = if r.short_name.is_empty() {
                                                r.institution_key.clone()
                                            } else {
                                                r.short_name.clone()
                                            };
                                            let title = {
                                                let mut t = r
                                                    .last_message
                                                    .clone()
                                                    .unwrap_or_else(|| r.status.clone());
                                                if r.rows_n > 0 {
                                                    t = format!("{} · {} rows", t, r.rows_n);
                                                }
                                                t
                                            };
                                            let time_label =
                                                r.received_hm.clone().unwrap_or_default();
                                            let status_cls = r.status.clone();
                                            view! {
                                                <li
                                                    class=format!("IngestCheck-item IngestCheck-item--{status_cls}")
                                                    title=title
                                                >
                                                    <span class="IngestCheck-mark">{mark}</span>
                                                    <span class="IngestCheck-name">{label}</span>
                                                    <span class="IngestCheck-time">{time_label}</span>
                                                </li>
                                            }
                                        })
                                        .collect_view()}
                                </ul>
                            }))
                        }
                    }
                })}
            </Suspense>
        </div>
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
// Deposits panel — bid-only (mail indications have no ask/spread)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum DepositCol {
    Term,
    Ccy,
    Institution,
    CnName,
    Bid,
    Note,
}

/// Three-state sort: unsorted (original) → ascending → descending → unsorted.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum DepositSort {
    #[default]
    None,
    Asc(DepositCol),
    Desc(DepositCol),
}

impl DepositSort {
    fn active_col(self) -> Option<DepositCol> {
        match self {
            DepositSort::None => None,
            DepositSort::Asc(c) | DepositSort::Desc(c) => Some(c),
        }
    }
}

/// Per-column value filters. Missing key = show all; present set = only those values.
#[derive(Clone, Default, PartialEq)]
struct DepositColFilters {
    map: HashMap<DepositCol, HashSet<String>>,
}

impl DepositColFilters {
    fn allows(&self, col: DepositCol, value: &str) -> bool {
        match self.map.get(&col) {
            None => true,
            Some(s) => s.contains(value),
        }
    }

    fn active(&self, col: DepositCol) -> bool {
        self.map.contains_key(&col)
    }

    fn clear_col(&mut self, col: DepositCol) {
        self.map.remove(&col);
    }
}

fn toggle_deposit_sort(sort: RwSignal<DepositSort>, col: DepositCol) {
    sort.update(|s| {
        *s = match *s {
            DepositSort::Asc(c) if c == col => DepositSort::Desc(col),
            DepositSort::Desc(c) if c == col => DepositSort::None,
            _ => DepositSort::Asc(col),
        };
    });
}

fn deposit_list_cell(q: &DepositQuoteView, col: DepositCol) -> String {
    match col {
        DepositCol::Term => q.term.clone(),
        DepositCol::Ccy => q.currency.clone(),
        DepositCol::Institution => q
            .institution_short_name
            .clone()
            .unwrap_or_else(|| q.institution_name.clone()),
        DepositCol::CnName => q.institution_cn_name.clone().unwrap_or_default(),
        DepositCol::Bid => fmt_opt(q.bid),
        DepositCol::Note => q.note.clone().unwrap_or_default(),
    }
}

fn deposit_best_cell(r: &DepositBestRow, col: DepositCol) -> String {
    match col {
        DepositCol::Term => r.term.clone(),
        DepositCol::Ccy => r.currency.clone(),
        DepositCol::Institution => r.institution.clone(),
        DepositCol::CnName => r.cn_name.clone(),
        DepositCol::Bid => fmt_opt(r.bid),
        DepositCol::Note => String::new(),
    }
}

fn unique_sorted(values: impl Iterator<Item = String>) -> Vec<String> {
    let mut v: Vec<String> = values.collect();
    v.sort();
    v.dedup();
    v
}

fn filter_deposit_list(rows: &[DepositQuoteView], filters: &DepositColFilters) -> Vec<DepositQuoteView> {
    rows.iter()
        .filter(|q| {
            [
                DepositCol::Term,
                DepositCol::Ccy,
                DepositCol::Institution,
                DepositCol::CnName,
                DepositCol::Bid,
                DepositCol::Note,
            ]
            .into_iter()
            .all(|col| filters.allows(col, &deposit_list_cell(q, col)))
        })
        .cloned()
        .collect()
}

fn filter_deposit_best(rows: &[DepositBestRow], filters: &DepositColFilters) -> Vec<DepositBestRow> {
    rows.iter()
        .filter(|r| {
            [
                DepositCol::Term,
                DepositCol::Ccy,
                DepositCol::Institution,
                DepositCol::CnName,
                DepositCol::Bid,
            ]
            .into_iter()
            .all(|col| filters.allows(col, &deposit_best_cell(r, col)))
        })
        .cloned()
        .collect()
}

fn toggle_filter_value(
    filters: RwSignal<DepositColFilters>,
    col: DepositCol,
    value: String,
    all_options: &[String],
) {
    let all: HashSet<String> = all_options.iter().cloned().collect();
    filters.update(|f| {
        let mut set = f.map.get(&col).cloned().unwrap_or_else(|| all.clone());
        if !set.remove(&value) {
            set.insert(value);
        }
        if set.is_empty() || set.len() == all.len() {
            f.clear_col(col);
        } else {
            f.map.insert(col, set);
        }
    });
}

#[component]
fn DepositColTh(
    label: &'static str,
    col: DepositCol,
    sort: RwSignal<DepositSort>,
    filters: RwSignal<DepositColFilters>,
    open_filter: RwSignal<Option<DepositCol>>,
    options: Vec<String>,
    #[prop(optional)] num: bool,
) -> impl IntoView {
    let options = StoredValue::new(options);
    let class = move || {
        let s = sort.get();
        let f = filters.get();
        let mut cls = String::from("Quotes-th-sort");
        if num {
            cls.push_str(" num");
        }
        if s.active_col() == Some(col) {
            cls.push_str(" is-active");
        }
        if f.active(col) {
            cls.push_str(" is-filtered");
        }
        cls
    };
    let arrow = move || match sort.get() {
        DepositSort::Asc(c) if c == col => " ↑",
        DepositSort::Desc(c) if c == col => " ↓",
        _ => " ↕",
    };
    view! {
        <th class=class>
            <div class="Quotes-th-head">
                <button
                    type="button"
                    class="Quotes-th-btn"
                    on:click=move |_| toggle_deposit_sort(sort, col)
                >
                    {label}
                    <span class="Quotes-th-arrow">{arrow}</span>
                </button>
                <button
                    type="button"
                    class=move || {
                        let mut c = String::from("Quotes-th-filter");
                        if filters.get().active(col) {
                            c.push_str(" is-on");
                        }
                        if open_filter.get() == Some(col) {
                            c.push_str(" is-open");
                        }
                        c
                    }
                    title="筛选"
                    aria-label="筛选"
                    on:click=move |_| {
                        open_filter.update(|o| {
                            *o = if *o == Some(col) { None } else { Some(col) };
                        });
                    }
                >
                    "▾"
                </button>
            </div>
            {move || {
                (open_filter.get() == Some(col)).then(|| {
                    let opts = options.get_value();
                    view! {
                        <div class="Quotes-th-panel" on:click=|ev| ev.stop_propagation()>
                            <div class="Quotes-th-panel-actions">
                                <button
                                    type="button"
                                    class="Quotes-th-panel-btn"
                                    on:click=move |_| {
                                        filters.update(|f| f.clear_col(col));
                                    }
                                >
                                    "全部"
                                </button>
                                <button
                                    type="button"
                                    class="Quotes-th-panel-btn"
                                    on:click=move |_| open_filter.set(None)
                                >
                                    "关闭"
                                </button>
                            </div>
                            <div class="Quotes-th-panel-list">
                                {opts
                                    .into_iter()
                                    .map(|val| {
                                        let val_check = val.clone();
                                        let val_toggle = val.clone();
                                        let display = if val.is_empty() {
                                            "(空)".to_string()
                                        } else {
                                            val
                                        };
                                        view! {
                                            <label class="Quotes-th-panel-item">
                                                <input
                                                    type="checkbox"
                                                    prop:checked=move || {
                                                        match filters.get().map.get(&col) {
                                                            None => true,
                                                            Some(s) => s.contains(&val_check),
                                                        }
                                                    }
                                                    on:change=move |_| {
                                                        let all = options.get_value();
                                                        toggle_filter_value(
                                                            filters,
                                                            col,
                                                            val_toggle.clone(),
                                                            &all,
                                                        );
                                                    }
                                                />
                                                <span>{display}</span>
                                            </label>
                                        }
                                    })
                                    .collect_view()}
                            </div>
                        </div>
                    }
                })
            }}
        </th>
    }
}

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
        let list_sort = RwSignal::new(DepositSort::default());
        let list_filters = RwSignal::new(DepositColFilters::default());
        let list_open_filter = RwSignal::new(None::<DepositCol>);
        let all_quotes = quotes.clone();
        let term_opts = unique_sorted(all_quotes.iter().map(|q| deposit_list_cell(q, DepositCol::Term)));
        let ccy_opts = unique_sorted(all_quotes.iter().map(|q| deposit_list_cell(q, DepositCol::Ccy)));
        let inst_opts =
            unique_sorted(all_quotes.iter().map(|q| deposit_list_cell(q, DepositCol::Institution)));
        let cn_opts =
            unique_sorted(all_quotes.iter().map(|q| deposit_list_cell(q, DepositCol::CnName)));
        let bid_opts = unique_sorted(all_quotes.iter().map(|q| deposit_list_cell(q, DepositCol::Bid)));
        let note_opts =
            unique_sorted(all_quotes.iter().map(|q| deposit_list_cell(q, DepositCol::Note)));
        Either::Right(view! {
            <DepositStatsStrip quotes=quotes.clone() filter_date />
            <DepositBestPanel quotes=quotes.clone() />
            <div class="Quotes-list">
                <div class="Quotes-best-title">"Deposit Quotes"</div>
                <div class="Quotes-table-wrap">
                    <table class="Quotes-table">
                        <thead>
                            <tr>
                                <DepositColTh label="Term" col=DepositCol::Term sort=list_sort filters=list_filters open_filter=list_open_filter options=term_opts />
                                <DepositColTh label="Ccy" col=DepositCol::Ccy sort=list_sort filters=list_filters open_filter=list_open_filter options=ccy_opts />
                                <DepositColTh label="Institution" col=DepositCol::Institution sort=list_sort filters=list_filters open_filter=list_open_filter options=inst_opts />
                                <DepositColTh label="中文名" col=DepositCol::CnName sort=list_sort filters=list_filters open_filter=list_open_filter options=cn_opts />
                                <DepositColTh label="Bid %" col=DepositCol::Bid sort=list_sort filters=list_filters open_filter=list_open_filter options=bid_opts num=true />
                                <DepositColTh label="Note" col=DepositCol::Note sort=list_sort filters=list_filters open_filter=list_open_filter options=note_opts />
                            </tr>
                        </thead>
                        <tbody>
                            {move || {
                                let s = list_sort.get();
                                let f = list_filters.get();
                                let mut rows = filter_deposit_list(&all_quotes, &f);
                                sort_deposit_quote_views(&mut rows, s);
                                rows.into_iter()
                                    .map(|q| {
                                        let term = q.term.clone();
                                        let short = q
                                            .institution_short_name
                                            .clone()
                                            .unwrap_or_else(|| q.institution_name.clone());
                                        let cn = q.institution_cn_name.clone().unwrap_or_default();
                                        let note = q.note.clone().unwrap_or_default();
                                        let best = is_best_deposit_bid(&all_quotes, &q);
                                        view! {
                                            <tr>
                                                <td class="Quotes-term">{term}</td>
                                                <td>{q.currency.clone()}</td>
                                                <td>
                                                    {short}
                                                    <span class="Quotes-sub">{q.institution_name.clone()}</span>
                                                </td>
                                                <td>{cn}</td>
                                                <td class={bid_cell_cls(best)}>{fmt_opt(q.bid)}</td>
                                                <td class="Quotes-note">{note}</td>
                                            </tr>
                                        }
                                    })
                                    .collect_view()
                            }}
                        </tbody>
                    </table>
                </div>
            </div>
            <p class="Quotes-legend">
                <span class="Quotes-tag Quotes-tag--best">BEST</span>
                " — highest bid for each term × currency. "
                "Note "
                <code>"indication"</code>
                " = bank indicative / reference rate (参考报价，非成交承诺). "
                "表头点击排序（无序→升→降）；▾ 按字段值筛选。"
            </p>
        })
    }
}

fn is_best_deposit_bid(all: &[DepositQuoteView], row: &DepositQuoteView) -> bool {
    let max_bid = all
        .iter()
        .filter(|r| r.term == row.term && r.currency == row.currency)
        .filter_map(|r| r.bid)
        .fold(f64::NAN, f64::max);
    row.bid.is_some_and(|b| b == max_bid)
}

fn sort_deposit_quote_views(rows: &mut [DepositQuoteView], s: DepositSort) {
    let (col, asc) = match s {
        DepositSort::None => return,
        DepositSort::Asc(c) => (c, true),
        DepositSort::Desc(c) => (c, false),
    };
    rows.sort_by(|a, b| {
        let ord = match col {
            DepositCol::Term => term_sort_key(&a.term)
                .cmp(&term_sort_key(&b.term))
                .then_with(|| a.term.cmp(&b.term)),
            DepositCol::Ccy => a.currency.cmp(&b.currency),
            DepositCol::Institution => {
                let sa = a
                    .institution_short_name
                    .as_deref()
                    .unwrap_or(a.institution_name.as_str());
                let sb = b
                    .institution_short_name
                    .as_deref()
                    .unwrap_or(b.institution_name.as_str());
                sa.cmp(sb)
            }
            DepositCol::CnName => a
                .institution_cn_name
                .as_deref()
                .unwrap_or("")
                .cmp(b.institution_cn_name.as_deref().unwrap_or("")),
            DepositCol::Bid => a
                .bid
                .partial_cmp(&b.bid)
                .unwrap_or(std::cmp::Ordering::Equal),
            DepositCol::Note => a
                .note
                .as_deref()
                .unwrap_or("")
                .cmp(b.note.as_deref().unwrap_or("")),
        };
        if asc {
            ord
        } else {
            ord.reverse()
        }
    });
}

#[component]
fn DepositStatsStrip(
    quotes: Vec<DepositQuoteView>,
    filter_date: Option<NaiveDate>,
) -> impl IntoView {
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
    let stats = [
        (
            "DATE",
            date.map(|d| d.format("%Y-%m-%d").to_string())
                .unwrap_or_else(|| "—".into()),
        ),
        ("INSTITUTIONS", institutions.to_string()),
        ("TERMS", terms.to_string()),
        ("LINES", quotes.len().to_string()),
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

/// Best bid for each (term, currency).
#[derive(Clone, Debug)]
struct DepositBestRow {
    term: String,
    currency: String,
    institution: String,
    cn_name: String,
    bid: Option<f64>,
}

fn term_sort_key(term: &str) -> i32 {
    match term {
        "O/N" => 0,
        "T/N" => 1,
        "1W" => 2,
        "2W" => 3,
        "3W" => 4,
        "1M" => 5,
        "2M" => 6,
        "3M" => 7,
        "4M" => 8,
        "5M" => 9,
        "6M" => 10,
        "9M" => 11,
        "1Y" => 12,
        "18M" => 13,
        "2Y" => 14,
        "3Y" => 15,
        "5Y" => 16,
        "10Y" => 17,
        _ => 100,
    }
}

fn aggregate_deposit_best(quotes: &[DepositQuoteView]) -> Vec<DepositBestRow> {
    let mut groups: HashMap<(String, String), Vec<&DepositQuoteView>> = HashMap::new();
    for q in quotes {
        groups
            .entry((q.term.clone(), q.currency.clone()))
            .or_default()
            .push(q);
    }
    let mut keys: Vec<(String, String)> = groups.keys().cloned().collect();
    keys.sort_by(|a, b| {
        a.1.cmp(&b.1)
            .then_with(|| term_sort_key(&a.0).cmp(&term_sort_key(&b.0)))
            .then_with(|| a.0.cmp(&b.0))
    });
    keys.into_iter()
        .map(|(term, currency)| {
            let rows = groups.remove(&(term.clone(), currency.clone())).unwrap_or_default();
            let winner = rows
                .iter()
                .filter(|r| r.bid.is_some())
                .max_by(|a, b| a.bid.partial_cmp(&b.bid).unwrap_or(std::cmp::Ordering::Equal));
            match winner {
                Some(r) => DepositBestRow {
                    term,
                    currency,
                    institution: r
                        .institution_short_name
                        .clone()
                        .unwrap_or_else(|| r.institution_name.clone()),
                    cn_name: r.institution_cn_name.clone().unwrap_or_default(),
                    bid: r.bid,
                },
                None => DepositBestRow {
                    term,
                    currency,
                    institution: "—".into(),
                    cn_name: String::new(),
                    bid: None,
                },
            }
        })
        .collect()
}

fn sort_deposit_best_rows(rows: &mut [DepositBestRow], s: DepositSort) {
    let (col, asc) = match s {
        DepositSort::None => return,
        DepositSort::Asc(c) => (c, true),
        DepositSort::Desc(c) => (c, false),
    };
    rows.sort_by(|a, b| {
        let ord = match col {
            DepositCol::Term => term_sort_key(&a.term)
                .cmp(&term_sort_key(&b.term))
                .then_with(|| a.term.cmp(&b.term)),
            DepositCol::Ccy => a.currency.cmp(&b.currency),
            DepositCol::Institution => a.institution.cmp(&b.institution),
            DepositCol::CnName => a.cn_name.cmp(&b.cn_name),
            DepositCol::Bid => a
                .bid
                .partial_cmp(&b.bid)
                .unwrap_or(std::cmp::Ordering::Equal),
            DepositCol::Note => std::cmp::Ordering::Equal,
        };
        if asc {
            ord
        } else {
            ord.reverse()
        }
    });
}

#[component]
fn DepositBestPanel(quotes: Vec<DepositQuoteView>) -> impl IntoView {
    let base_rows = aggregate_deposit_best(&quotes);
    let sort = RwSignal::new(DepositSort::default());
    let filters = RwSignal::new(DepositColFilters::default());
    let open_filter = RwSignal::new(None::<DepositCol>);
    let term_opts = unique_sorted(base_rows.iter().map(|r| deposit_best_cell(r, DepositCol::Term)));
    let ccy_opts = unique_sorted(base_rows.iter().map(|r| deposit_best_cell(r, DepositCol::Ccy)));
    let inst_opts =
        unique_sorted(base_rows.iter().map(|r| deposit_best_cell(r, DepositCol::Institution)));
    let cn_opts = unique_sorted(base_rows.iter().map(|r| deposit_best_cell(r, DepositCol::CnName)));
    let bid_opts = unique_sorted(base_rows.iter().map(|r| deposit_best_cell(r, DepositCol::Bid)));
    view! {
        <div class="Quotes-best">
            <div class="Quotes-best-title">
                <span class="Quotes-tag Quotes-tag--best">Best Deposit Quote</span>
                " — highest bid per term × currency"
            </div>
            <div class="Quotes-table-wrap">
                <table class="Quotes-table Quotes-table--best">
                    <thead>
                        <tr>
                            <DepositColTh label="Term" col=DepositCol::Term sort=sort filters=filters open_filter=open_filter options=term_opts />
                            <DepositColTh label="Ccy" col=DepositCol::Ccy sort=sort filters=filters open_filter=open_filter options=ccy_opts />
                            <DepositColTh label="Institution" col=DepositCol::Institution sort=sort filters=filters open_filter=open_filter options=inst_opts />
                            <DepositColTh label="中文名" col=DepositCol::CnName sort=sort filters=filters open_filter=open_filter options=cn_opts />
                            <DepositColTh label="Bid %" col=DepositCol::Bid sort=sort filters=filters open_filter=open_filter options=bid_opts num=true />
                        </tr>
                    </thead>
                    <tbody>
                        {move || {
                            let s = sort.get();
                            let f = filters.get();
                            let mut rows = filter_deposit_best(&base_rows, &f);
                            sort_deposit_best_rows(&mut rows, s);
                            rows.into_iter()
                                .map(|r| {
                                    view! {
                                        <tr>
                                            <td class="Quotes-term">{r.term}</td>
                                            <td>{r.currency}</td>
                                            <td class="Quotes-bester">{r.institution}</td>
                                            <td>{r.cn_name}</td>
                                            <td class="num Quotes-best-bid">{fmt_opt(r.bid)}</td>
                                        </tr>
                                    }
                                })
                                .collect_view()
                        }}
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
                                let inst_label = q
                                    .institution_short_name
                                    .clone()
                                    .unwrap_or_else(|| q.institution_name.clone());
                                let cn = q.institution_cn_name.clone().unwrap_or_default();
                                let best = best_bond(&quotes, &q);
                                view! {
                                    <tr>
                                        <td class="Quotes-term">{code}</td>
                                        <td>{q.bond_name.clone().unwrap_or_default()}</td>
                                        <td class="num">{fmt_date(q.maturity_date)}</td>
                                        <td class="num">{fmt_opt(q.coupon)}</td>
                                        <td>
                                            {inst_label}
                                            <span class="Quotes-sub">{cn}</span>
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

fn institution_filter_label(inst: &InstitutionOption) -> String {
    let short = inst
        .short_name
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or(inst.name.as_str());
    match inst.cn_name.as_deref().filter(|s| !s.is_empty()) {
        Some(cn) => format!("{short} · {cn}"),
        None => short.to_string(),
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

#[server(GetIngestDayChecks, "/api")]
pub async fn get_ingest_day_checks(
    quote_date: NaiveDate,
) -> Result<Vec<IngestDayCheckView>, ServerFnError> {
    let db = crate::server::db::use_db().await?;
    let rows = db
        .query(
            r#"
            SELECT
                COALESCE(c.institution_key, i.ingest_key, i.id) AS institution_key,
                COALESCE(i.short_name, c.institution_key, i.name, '?') AS short_name,
                i.cn_name,
                COALESCE(c.status, CASE WHEN q.has_quotes THEN 'ok' ELSE 'pending' END) AS status,
                COALESCE(c.rows_n, q.cnt, 0)::int AS rows_n,
                COALESCE(c.mail_count, 0)::int AS mail_count,
                c.last_message,
                to_char(
                  COALESCE(c.mail_received_at, c.checked_at),
                  'HH24:MI'
                ) AS received_hm
            FROM "Institution" i
            LEFT JOIN "IngestDayCheck" c
              ON c.institution_id = i.id
             AND c.quote_date = $1
             AND c.business = 'deposit'
            LEFT JOIN LATERAL (
              SELECT COUNT(*)::int AS cnt, COUNT(*) > 0 AS has_quotes
              FROM "DepositQuote" d
              WHERE d.institution_id = i.id AND d.quote_date = $1
            ) q ON true
            WHERE i.is_active = true
              AND (i.ingest_key IS NOT NULL OR c.id IS NOT NULL OR q.has_quotes)
            ORDER BY
              COALESCE(c.mail_received_at, c.checked_at) ASC NULLS LAST,
              COALESCE(i.short_name, i.name)
            "#,
            &[&quote_date],
        )
        .await
        .map_err(|e| lib::emsg(e, "Ingest day checks"))?;
    Ok(rows
        .into_iter()
        .map(|r| IngestDayCheckView {
            institution_key: r.get(0),
            short_name: r.get(1),
            cn_name: r.get(2),
            status: r.get(3),
            rows_n: r.get(4),
            mail_count: r.get(5),
            last_message: r.get(6),
            received_hm: r.get(7),
        })
        .collect())
}

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
            cn_name: r.cn_name,
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
            institution_cn_name: r.institution_cn_name,
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
            institution_cn_name: r.institution_cn_name,
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

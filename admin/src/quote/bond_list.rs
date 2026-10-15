use chrono::NaiveDate;
use leptos::{either::Either, prelude::*};
use leptos_meta::Title;
use leptos_router::components::A;
use serde::{Deserialize, Serialize};

use crate::util::{AlertDanger, Loading};
use std::convert::TryInto;

#[component]
pub fn BondQuoteList() -> impl IntoView {
    let from = RwSignal::new(String::new());
    let to = RwSignal::new(String::new());
    let institution_id = RwSignal::new(String::new());

    let filter = move || {
        (
            parse_date(&from.get()),
            parse_date(&to.get()),
            trim_opt(&institution_id.get()),
        )
    };
    let quotes = Resource::new_blocking(
        move || filter(),
        move |(from, to, institution_id)| async move {
            get_bond_quotes(from, to, institution_id).await
        },
    );

    view! {
        <Title text="Bond quotes" />
        <h1>
            <span>"Bond quotes"</span>
            <span>" "</span>
            <A href="/quote/bond/new">"+1"</A>
        </h1>

        <form class="Card filter" on:submit=move |ev| ev.prevent_default()>
            <label>
                <div>From</div>
                <input
                    type="date"
                    prop:value=move || from.get()
                    on:input=move |ev| from.set(event_target_value(&ev))
                />
            </label>
            <label>
                <div>To</div>
                <input
                    type="date"
                    prop:value=move || to.get()
                    on:input=move |ev| to.set(event_target_value(&ev))
                />
            </label>
            <crate::institution::InstitutionSelect selected=institution_id.get_untracked() />
        </form>

        <Suspense fallback=move || {
            view! { <Loading /> }
        }>
            {move || Suspend::new(async move {
                match quotes.await {
                    Err(e) => Either::Left(view! { <AlertDanger text=e.to_string() /> }),
                    Ok(quotes) => Either::Right(view! { <BondQuoteListItems quotes /> }),
                }
            })}
        </Suspense>
    }
    .into_any()
}

#[component]
pub fn BondQuoteListItems(quotes: Vec<BondQuoteListItem>) -> impl IntoView {
    if quotes.is_empty() {
        Either::Left(view! { <p>"0 bond quotes"</p> })
    } else {
        Either::Right(view! {
            <div class="Quotes-table-wrap">
                <table class="Quotes-table">
                    <thead>
                        <tr>
                            <th>Date</th>
                            <th>Code</th>
                            <th>Name</th>
                            <th>Ccy</th>
                            <th>Maturity</th>
                            <th>Institution</th>
                            <th>Bid Yld</th>
                            <th>Ask Yld</th>
                            <th>Bid Px</th>
                            <th>Ask Px</th>
                            <th>Dir</th>
                        </tr>
                    </thead>
                    <tbody>
                        {quotes
                            .into_iter()
                            .map(|q| {
                                view! {
                                    <tr>
                                        <td><A href=format!("/quote/bond/{}", q.id)>{q.quote_date.format("%Y-%m-%d").to_string()}</A></td>
                                        <td>{q.bond_code}</td>
                                        <td>{q.bond_name.unwrap_or_default()}</td>
                                        <td>{q.currency}</td>
                                        <td class="num">{fmt_date(q.maturity_date)}</td>
                                        <td>{q.institution_name}</td>
                                        <td class="num">{fmt_dec(q.bid_yield)}</td>
                                        <td class="num">{fmt_dec(q.ask_yield)}</td>
                                        <td class="num">{fmt_dec(q.bid_price)}</td>
                                        <td class="num">{fmt_dec(q.ask_price)}</td>
                                        <td>{q.direction.unwrap_or_default()}</td>
                                    </tr>
                                }
                            })
                            .collect_view()}
                    </tbody>
                </table>
            </div>
        })
    }
}

#[server(GetBondQuoteList, "/api")]
async fn get_bond_quotes(
    from: Option<NaiveDate>,
    to: Option<NaiveDate>,
    institution_id: Option<String>,
) -> Result<Vec<BondQuoteListItem>, ServerFnError> {
    let db = crate::server::db::use_db().await?;
    let rows = cornucopia::queries::admin_quote::bond_list()
        .bind(&db, &from, &to, &institution_id)
        .all()
        .await
        .map_err(|e| lib::emsg(e, "Bond quotes list"))?;
    Ok(rows
        .into_iter()
        .map(|r| BondQuoteListItem {
            id: r.id,
            quote_date: r.quote_date,
            bond_code: r.bond_code,
            bond_name: r.bond_name,
            currency: r.currency,
            maturity_date: r.maturity_date,
            institution_name: r.institution_name,
            bid_yield: r.bid_yield,
            ask_yield: r.ask_yield,
            bid_price: r.bid_price,
            ask_price: r.ask_price,
            direction: r.direction,
        })
        .collect())
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BondQuoteListItem {
    pub id: String,
    pub quote_date: NaiveDate,
    pub bond_code: String,
    pub bond_name: Option<String>,
    pub currency: String,
    pub maturity_date: Option<NaiveDate>,
    pub institution_name: String,
    pub bid_yield: Option<rust_decimal::Decimal>,
    pub ask_yield: Option<rust_decimal::Decimal>,
    pub bid_price: Option<rust_decimal::Decimal>,
    pub ask_price: Option<rust_decimal::Decimal>,
    pub direction: Option<String>,
}

fn fmt_dec(d: Option<rust_decimal::Decimal>) -> String {
    d.and_then(|d| d.try_into().ok())
        .map(|x: f64| format!("{x:.4}"))
        .unwrap_or_else(|| "—".into())
}

fn fmt_date(d: Option<NaiveDate>) -> String {
    d.map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "—".into())
}

fn parse_date(s: &str) -> Option<NaiveDate> {
    let t = s.trim();
    if t.is_empty() {
        return None;
    }
    NaiveDate::parse_from_str(t, "%Y-%m-%d").ok()
}

fn trim_opt(s: &str) -> Option<String> {
    let t = s.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}

use chrono::NaiveDate;
use leptos::{either::Either, prelude::*};
use leptos_meta::Title;
use leptos_router::components::A;
use serde::{Deserialize, Serialize};

use crate::util::{AlertDanger, Loading};

#[component]
pub fn DepositQuoteList() -> impl IntoView {
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
            get_deposit_quotes(from, to, institution_id).await
        },
    );

    view! {
        <Title text="Deposit quotes" />
        <h1>
            <span>"Deposit quotes"</span>
            <span>" "</span>
            <A href="/quote/deposit/new">"+1"</A>
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
                    Ok(quotes) => Either::Right(view! { <DepositQuoteListItems quotes /> }),
                }
            })}
        </Suspense>
    }
    .into_any()
}

#[component]
pub fn DepositQuoteListItems(quotes: Vec<DepositQuoteListItem>) -> impl IntoView {
    if quotes.is_empty() {
        Either::Left(view! { <p>"0 deposit quotes"</p> })
    } else {
        Either::Right(view! {
            <div class="Quotes-table-wrap">
                <table class="Quotes-table">
                    <thead>
                        <tr>
                            <th>Date</th>
                            <th>Institution</th>
                            <th>Term</th>
                            <th>Currency</th>
                            <th>Bid %</th>
                            <th>Ask %</th>
                            <th>Note</th>
                        </tr>
                    </thead>
                    <tbody>
                        {quotes
                            .into_iter()
                            .map(|q| {
                                view! {
                                    <tr>
                                        <td><A href=format!("/quote/deposit/{}", q.id)>{q.quote_date.format("%Y-%m-%d").to_string()}</A></td>
                                        <td>{q.institution_name}</td>
                                        <td>{q.term}</td>
                                        <td>{q.currency}</td>
                                        <td class="num">{fmt_dec(q.bid_rate)}</td>
                                        <td class="num">{fmt_dec(q.ask_rate)}</td>
                                        <td>{q.note.unwrap_or_default()}</td>
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

#[server(GetDepositQuoteList, "/api")]
async fn get_deposit_quotes(
    from: Option<NaiveDate>,
    to: Option<NaiveDate>,
    institution_id: Option<String>,
) -> Result<Vec<DepositQuoteListItem>, ServerFnError> {
    let db = crate::server::db::use_db().await?;
    let rows = cornucopia::queries::admin_quote::deposit_list()
        .bind(&db, &from, &to, &institution_id)
        .all()
        .await
        .map_err(|e| lib::emsg(e, "Deposit quotes list"))?;
    Ok(rows
        .into_iter()
        .map(|r| DepositQuoteListItem {
            id: r.id,
            quote_date: r.quote_date,
            institution_name: r.institution_name,
            term: r.term,
            currency: r.currency,
            bid_rate: r.bid_rate,
            ask_rate: r.ask_rate,
            note: r.note,
        })
        .collect())
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DepositQuoteListItem {
    pub id: String,
    pub quote_date: NaiveDate,
    pub institution_name: String,
    pub term: String,
    pub currency: String,
    pub bid_rate: Option<rust_decimal::Decimal>,
    pub ask_rate: Option<rust_decimal::Decimal>,
    pub note: Option<String>,
}

fn fmt_dec(d: Option<rust_decimal::Decimal>) -> String {
    use std::convert::TryInto;
    d.and_then(|d| d.try_into().ok())
        .map(|x: f64| format!("{x:.4}"))
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

use chrono::NaiveDate;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use super::QuoteError;
use crate::form::FormFooter;

#[derive(Default, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DepositQuoteFormData {
    pub id: String,
    pub quote_date: NaiveDate,
    pub institution_id: String,
    pub term: String,
    pub term_sort: i32,
    pub currency: String,
    pub bid_rate: Option<String>,
    pub ask_rate: Option<String>,
    pub note: Option<String>,
}

#[component]
pub fn DepositQuoteForm(form: DepositQuoteFormData) -> impl IntoView {
    let deposit_update = ServerAction::<DepositQuoteUpdate>::new();
    let pending = deposit_update.pending();

    let quote_date = form.quote_date.format("%Y-%m-%d").to_string();
    let bid = form.bid_rate.clone().unwrap_or_default();
    let ask = form.ask_rate.clone().unwrap_or_default();
    let note = form.note.clone().unwrap_or_default();
    let institution_id = form.institution_id.clone();

    view! {
        <ActionForm action=deposit_update>
            <fieldset prop:disabled=move || pending()>
                <legend>Data</legend>
                <input type="hidden" name="id" value=form.id />
                <div class="Grid-fluid-2">
                    <label>
                        <div>Quote date</div>
                        <input
                            type="date"
                            name="quote_date"
                            value=quote_date
                            required
                        />
                    </label>
                    <crate::institution::InstitutionSelect selected=institution_id />
                    <label>
                        <div>Term</div>
                        <input type="text" name="term" value=form.term.clone() required />
                    </label>
                    <label>
                        <div>Term sort</div>
                        <input
                            type="number"
                            name="term_sort"
                            value=form.term_sort.to_string()
                            required
                        />
                    </label>
                    <label>
                        <div>Currency</div>
                        <input type="text" name="currency" value=form.currency.clone() required />
                    </label>
                    <label>
                        <div>Bid rate %</div>
                        <input type="number" name="bid_rate" value=bid step="0.0001" />
                    </label>
                    <label>
                        <div>Ask rate %</div>
                        <input type="number" name="ask_rate" value=ask step="0.0001" />
                    </label>
                    <label class="full">
                        <div>Note</div>
                        <textarea name="note" rows="2">{note}</textarea>
                    </label>
                </div>
                <FormFooter action=deposit_update submit_text="Update deposit quote" />
            </fieldset>
        </ActionForm>
    }
}

#[server(DepositQuoteUpdate, "/api")]
pub async fn deposit_quote_update(
    id: String,
    quote_date: NaiveDate,
    institution_id: String,
    term: String,
    term_sort: i32,
    currency: String,
    bid_rate: Option<String>,
    ask_rate: Option<String>,
    note: Option<String>,
) -> Result<Result<(), QuoteError>, ServerFnError> {
    let db = crate::server::db::use_db().await?;
    let bid = parse_rate(bid_rate)?;
    let ask = parse_rate(ask_rate)?;
    let note = note.and_then(|n| {
        let t = n.trim();
        if t.is_empty() {
            None
        } else {
            Some(t.to_string())
        }
    });
    let term = term.trim().to_string();
    let currency = currency.trim().to_string();

    cornucopia::queries::admin_quote::deposit_update()
        .bind(&db, &quote_date, &institution_id, &term, &term_sort, &currency, &bid, &ask, &note, &id)
        .opt()
        .await
        .map_err(|e| lib::emsg(e, "Deposit quote update"))?;
    Ok(Ok(()))
}

pub(crate) fn parse_rate(s: Option<String>) -> Result<Option<rust_decimal::Decimal>, ServerFnError> {
    match s {
        Some(s) => {
            let t = s.trim();
            if t.is_empty() {
                Ok(None)
            } else {
                t
                    .parse::<rust_decimal::Decimal>()
                    .map(Some)
                    .map_err(|e| ServerFnError::new(format!("invalid rate: {e}")))
            }
        }
        None => Ok(None),
    }
}

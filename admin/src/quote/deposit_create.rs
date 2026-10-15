use chrono::NaiveDate;
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_navigate;

use super::{parse_rate, QuoteError};
use crate::form::FormFooter;

#[component]
pub fn DepositQuoteNew() -> impl IntoView {
    let create = ServerAction::<DepositQuoteCreate>::new();
    let value = create.value();
    let pending = create.pending();
    Effect::new(move |_| {
        let v = value.get();
        if let Some(v) = v {
            let create_result = v.map_err(|_| QuoteError::ServerError).flatten();
            if let Ok(id) = create_result {
                tracing::info!("navigate deposit quote create_result ok");
                let navigate = use_navigate();
                let to = format!("/quote/deposit/{}", id);
                navigate(&to, Default::default());
            }
        }
    });
    view! {
        <Title text="New deposit quote" />
        <h1>"New deposit quote"</h1>
        <ActionForm action=create>
            <fieldset prop:disabled=move || pending()>
                <legend>Data</legend>
                <div class="Grid-fluid-2">
                    <label>
                        <div>Quote date</div>
                        <input type="date" name="quote_date" required />
                    </label>
                    <crate::institution::InstitutionSelect />
                    <label>
                        <div>Term</div>
                        <input type="text" name="term" placeholder="1M" required />
                    </label>
                    <label>
                        <div>Term sort</div>
                        <input type="number" name="term_sort" value="0" required />
                    </label>
                    <label>
                        <div>Currency</div>
                        <input type="text" name="currency" value="USD" required />
                    </label>
                    <label>
                        <div>Bid rate %</div>
                        <input type="number" name="bid_rate" step="0.0001" />
                    </label>
                    <label>
                        <div>Ask rate %</div>
                        <input type="number" name="ask_rate" step="0.0001" />
                    </label>
                    <label class="full">
                        <div>Note</div>
                        <textarea name="note" rows="2"></textarea>
                    </label>
                </div>
                <FormFooter action=create submit_text="Create deposit quote" />
            </fieldset>
        </ActionForm>
    }
    .into_any()
}

#[server(DepositQuoteCreate, "/api")]
pub async fn deposit_quote_create(
    quote_date: NaiveDate,
    institution_id: String,
    term: String,
    term_sort: i32,
    currency: String,
    bid_rate: Option<String>,
    ask_rate: Option<String>,
    note: Option<String>,
) -> Result<Result<String, QuoteError>, ServerFnError> {
    let db = crate::server::db::use_db().await?;
    let id = cuid2::create_id();

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

    let created = cornucopia::queries::admin_quote::deposit_create()
        .bind(&db, &id, &quote_date, &institution_id, &term, &term_sort, &currency, &bid, &ask, &note)
        .one()
        .await;
    match created {
        Ok(_) => Ok(Ok(id)),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("DepositQuote_inst_date_term_key") {
                Ok(Err(QuoteError::Duplicate))
            } else {
                Err(lib::emsg(e, "Deposit quote create"))
            }
        }
    }
}

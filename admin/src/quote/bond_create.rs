use chrono::NaiveDate;
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_navigate;

use super::{parse_rate, trim_str_opt, QuoteError};
use crate::form::FormFooter;

#[component]
pub fn BondQuoteNew() -> impl IntoView {
    let create = ServerAction::<BondQuoteCreate>::new();
    let value = create.value();
    let pending = create.pending();
    Effect::new(move |_| {
        let v = value.get();
        if let Some(v) = v {
            let create_result = v.map_err(|_| QuoteError::ServerError).flatten();
            if let Ok(id) = create_result {
                tracing::info!("navigate bond quote create_result ok");
                let navigate = use_navigate();
                let to = format!("/quote/bond/{}", id);
                navigate(&to, Default::default());
            }
        }
    });
    view! {
        <Title text="New bond quote" />
        <h1>"New bond quote"</h1>
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
                        <div>Bond code</div>
                        <input type="text" name="bond_code" required />
                    </label>
                    <label>
                        <div>Bond name</div>
                        <input type="text" name="bond_name" />
                    </label>
                    <label>
                        <div>Currency</div>
                        <input type="text" name="currency" value="USD" required />
                    </label>
                    <label>
                        <div>Maturity date</div>
                        <input type="date" name="maturity_date" />
                    </label>
                    <label>
                        <div>Coupon %</div>
                        <input type="number" name="coupon_rate" step="0.0001" />
                    </label>
                    <label>
                        <div>Bid yield</div>
                        <input type="number" name="bid_yield" step="0.0001" />
                    </label>
                    <label>
                        <div>Ask yield</div>
                        <input type="number" name="ask_yield" step="0.0001" />
                    </label>
                    <label>
                        <div>Bid price</div>
                        <input type="number" name="bid_price" step="0.0001" />
                    </label>
                    <label>
                        <div>Ask price</div>
                        <input type="number" name="ask_price" step="0.0001" />
                    </label>
                    <label>
                        <div>Direction</div>
                        <input type="text" name="direction" placeholder="Bid / Ask / Both" />
                    </label>
                    <label class="full">
                        <div>Note</div>
                        <textarea name="note" rows="2"></textarea>
                    </label>
                </div>
                <FormFooter action=create submit_text="Create bond quote" />
            </fieldset>
        </ActionForm>
    }
    .into_any()
}

#[server(BondQuoteCreate, "/api")]
pub async fn bond_quote_create(
    quote_date: NaiveDate,
    institution_id: String,
    bond_code: String,
    bond_name: Option<String>,
    currency: String,
    maturity_date: Option<NaiveDate>,
    coupon_rate: Option<String>,
    bid_yield: Option<String>,
    ask_yield: Option<String>,
    bid_price: Option<String>,
    ask_price: Option<String>,
    direction: Option<String>,
    note: Option<String>,
) -> Result<Result<String, QuoteError>, ServerFnError> {
    let db = crate::server::db::use_db().await?;
    let id = cuid2::create_id();

    let bond_name = trim_str_opt(bond_name);
    let direction = trim_str_opt(direction);
    let note = trim_str_opt(note);
    let bond_code = bond_code.trim().to_string();
    let currency = currency.trim().to_string();
    let coupon_rate = parse_rate(coupon_rate)?;
    let bid_yield = parse_rate(bid_yield)?;
    let ask_yield = parse_rate(ask_yield)?;
    let bid_price = parse_rate(bid_price)?;
    let ask_price = parse_rate(ask_price)?;

    let created = cornucopia::queries::admin_quote::bond_create()
        .bind(
            &db,
            &id,
            &quote_date,
            &institution_id,
            &bond_code,
            &bond_name,
            &currency,
            &maturity_date,
            &coupon_rate,
            &bid_yield,
            &ask_yield,
            &bid_price,
            &ask_price,
            &direction,
            &note,
        )
        .one()
        .await;
    match created {
        Ok(_) => Ok(Ok(id)),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("BondQuote_inst_date_code_key") {
                Ok(Err(QuoteError::Duplicate))
            } else {
                Err(lib::emsg(e, "Bond quote create"))
            }
        }
    }
}

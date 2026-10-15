use chrono::NaiveDate;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use super::{parse_rate, QuoteError};
use crate::form::FormFooter;

#[derive(Default, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BondQuoteFormData {
    pub id: String,
    pub quote_date: NaiveDate,
    pub institution_id: String,
    pub bond_code: String,
    pub bond_name: Option<String>,
    pub currency: String,
    pub maturity_date: Option<NaiveDate>,
    pub coupon_rate: Option<String>,
    pub bid_yield: Option<String>,
    pub ask_yield: Option<String>,
    pub bid_price: Option<String>,
    pub ask_price: Option<String>,
    pub direction: Option<String>,
    pub note: Option<String>,
}

#[component]
pub fn BondQuoteForm(form: BondQuoteFormData) -> impl IntoView {
    let update = ServerAction::<BondQuoteUpdate>::new();
    let pending = update.pending();

    let quote_date = form.quote_date.format("%Y-%m-%d").to_string();
    let maturity = form
        .maturity_date
        .map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_default();
    let institution_id = form.institution_id.clone();
    let bond_name = form.bond_name.clone().unwrap_or_default();
    let coupon = form.coupon_rate.clone().unwrap_or_default();
    let bid_yield = form.bid_yield.clone().unwrap_or_default();
    let ask_yield = form.ask_yield.clone().unwrap_or_default();
    let bid_price = form.bid_price.clone().unwrap_or_default();
    let ask_price = form.ask_price.clone().unwrap_or_default();
    let direction = form.direction.clone().unwrap_or_default();
    let note = form.note.clone().unwrap_or_default();

    view! {
        <ActionForm action=update>
            <fieldset prop:disabled=move || pending()>
                <legend>Data</legend>
                <input type="hidden" name="id" value=form.id />
                <div class="Grid-fluid-2">
                    <label>
                        <div>Quote date</div>
                        <input type="date" name="quote_date" value=quote_date required />
                    </label>
                    <crate::institution::InstitutionSelect selected=institution_id />
                    <label>
                        <div>Bond code</div>
                        <input type="text" name="bond_code" value=form.bond_code.clone() required />
                    </label>
                    <label>
                        <div>Bond name</div>
                        <input type="text" name="bond_name" value=bond_name />
                    </label>
                    <label>
                        <div>Currency</div>
                        <input type="text" name="currency" value=form.currency.clone() required />
                    </label>
                    <label>
                        <div>Maturity date</div>
                        <input type="date" name="maturity_date" value=maturity />
                    </label>
                    <label>
                        <div>Coupon %</div>
                        <input type="number" name="coupon_rate" value=coupon step="0.0001" />
                    </label>
                    <label>
                        <div>Bid yield</div>
                        <input type="number" name="bid_yield" value=bid_yield step="0.0001" />
                    </label>
                    <label>
                        <div>Ask yield</div>
                        <input type="number" name="ask_yield" value=ask_yield step="0.0001" />
                    </label>
                    <label>
                        <div>Bid price</div>
                        <input type="number" name="bid_price" value=bid_price step="0.0001" />
                    </label>
                    <label>
                        <div>Ask price</div>
                        <input type="number" name="ask_price" value=ask_price step="0.0001" />
                    </label>
                    <label>
                        <div>Direction</div>
                        <input type="text" name="direction" value=direction placeholder="Bid / Ask / Both" />
                    </label>
                    <label class="full">
                        <div>Note</div>
                        <textarea name="note" rows="2">{note}</textarea>
                    </label>
                </div>
                <FormFooter action=update submit_text="Update bond quote" />
            </fieldset>
        </ActionForm>
    }
}

#[server(BondQuoteUpdate, "/api")]
pub async fn bond_quote_update(
    id: String,
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
) -> Result<Result<(), QuoteError>, ServerFnError> {
    let db = crate::server::db::use_db().await?;
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

    cornucopia::queries::admin_quote::bond_update()
        .bind(
            &db,
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
            &id,
        )
        .opt()
        .await
        .map_err(|e| lib::emsg(e, "Bond quote update"))?;
    Ok(Ok(()))
}

pub(crate) fn trim_str_opt(s: Option<String>) -> Option<String> {
    s.and_then(|n| {
        let t = n.trim();
        if t.is_empty() {
            None
        } else {
            Some(t.to_string())
        }
    })
}

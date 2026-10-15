use leptos::{either::EitherOf3, prelude::*};
use leptos_meta::Title;
use leptos_router::{hooks::use_params, params::Params};
use serde::{Deserialize, Serialize};

use super::{BondQuoteDeleteForm, BondQuoteFormData, BondQuoteForm, QuoteError};
use crate::util::{AlertDanger, Loading};

#[derive(Params, Clone, Debug, PartialEq, Eq)]
pub struct BondQuoteParams {
    id: String,
}
#[derive(Default, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BondQuotePageData {
    pub form: BondQuoteFormData,
}

#[component]
pub fn BondQuotePage() -> impl IntoView {
    let params = use_params::<BondQuoteParams>();
    let id = Memo::new(move |prev: Option<&Result<String, QuoteError>>| {
        params.with(|q| match q {
            Ok(q) => Ok(q.id.clone()),
            Err(_) => {
                if let Some(Ok(prev)) = prev {
                    Ok(prev.to_owned())
                } else {
                    Err(QuoteError::InvalidId)
                }
            }
        })
    });

    let quote = Resource::new_blocking(
        move || id(),
        move |id| async move {
            match id {
                Err(e) => Ok(Err(e)),
                Ok(id) => get_bond_quote(id).await,
            }
        },
    );

    view! {
        <Suspense fallback=move || {
            view! { <Loading /> }
        }>
            {move || Suspend::new(async move {
                match quote.await {
                    Ok(Ok(quote)) => EitherOf3::A(view! { <BondQuotePageView quote=quote /> }),
                    Ok(Err(e)) => EitherOf3::B(view! { <AlertDanger text=e.to_string() /> }),
                    Err(e) => EitherOf3::C(view! { <AlertDanger text=e.to_string() /> }),
                }
            })}
        </Suspense>
    }
    .into_any()
}

#[component]
pub fn BondQuotePageView(quote: BondQuotePageData) -> impl IntoView {
    view! {
        <Title text=format!("Bond quote: {}", quote.form.bond_code) />
        <section class="PostPage">
            <BondQuoteForm form=quote.form.clone() />
            <BondQuoteDeleteForm id=quote.form.id.clone() />
        </section>
    }
}

#[server(GetBondQuote, "/api")]
pub async fn get_bond_quote(
    id: String,
) -> Result<Result<BondQuotePageData, QuoteError>, ServerFnError> {
    let db = crate::server::db::use_db().await?;
    let quote = cornucopia::queries::admin_quote::bond_page()
        .bind(&db, &id)
        .opt()
        .await
        .map_err(|e| lib::emsg(e, "Bond quote find"))?;

    let Some(quote) = quote else {
        crate::server::serverr_404();
        return Ok(Err(QuoteError::NotFound));
    };

    let page = BondQuotePageData {
        form: BondQuoteFormData {
            id: quote.id,
            quote_date: quote.quote_date,
            institution_id: quote.institution_id,
            bond_code: quote.bond_code,
            bond_name: quote.bond_name,
            currency: quote.currency,
            maturity_date: quote.maturity_date,
            coupon_rate: quote.coupon_rate.map(|d| d.to_string()),
            bid_yield: quote.bid_yield.map(|d| d.to_string()),
            ask_yield: quote.ask_yield.map(|d| d.to_string()),
            bid_price: quote.bid_price.map(|d| d.to_string()),
            ask_price: quote.ask_price.map(|d| d.to_string()),
            direction: quote.direction,
            note: quote.note,
        },
    };
    Ok(Ok(page))
}

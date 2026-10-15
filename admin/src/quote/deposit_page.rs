use leptos::{either::EitherOf3, prelude::*};
use leptos_meta::Title;
use leptos_router::{hooks::use_params, params::Params};
use serde::{Deserialize, Serialize};

use super::{DepositQuoteDeleteForm, DepositQuoteFormData, DepositQuoteForm, QuoteError};
use crate::util::{AlertDanger, Loading};

#[derive(Params, Clone, Debug, PartialEq, Eq)]
pub struct DepositQuoteParams {
    id: String,
}
#[derive(Default, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DepositQuotePageData {
    pub form: DepositQuoteFormData,
}

#[component]
pub fn DepositQuotePage() -> impl IntoView {
    let params = use_params::<DepositQuoteParams>();
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
                Ok(id) => get_deposit_quote(id).await,
            }
        },
    );

    view! {
        <Suspense fallback=move || {
            view! { <Loading /> }
        }>
            {move || Suspend::new(async move {
                match quote.await {
                    Ok(Ok(quote)) => {
                        EitherOf3::A(view! { <DepositQuotePageView quote=quote /> })
                    }
                    Ok(Err(e)) => EitherOf3::B(view! { <AlertDanger text=e.to_string() /> }),
                    Err(e) => EitherOf3::C(view! { <AlertDanger text=e.to_string() /> }),
                }
            })}
        </Suspense>
    }
    .into_any()
}

#[component]
pub fn DepositQuotePageView(quote: DepositQuotePageData) -> impl IntoView {
    view! {
        <Title text=format!("Deposit quote: {}", quote.form.term) />
        <section class="PostPage">
            <DepositQuoteForm form=quote.form.clone() />
            <DepositQuoteDeleteForm id=quote.form.id.clone() />
        </section>
    }
}

#[server(GetDepositQuote, "/api")]
pub async fn get_deposit_quote(
    id: String,
) -> Result<Result<DepositQuotePageData, QuoteError>, ServerFnError> {
    let db = crate::server::db::use_db().await?;
    let quote = cornucopia::queries::admin_quote::deposit_page()
        .bind(&db, &id)
        .opt()
        .await
        .map_err(|e| lib::emsg(e, "Deposit quote find"))?;

    let Some(quote) = quote else {
        crate::server::serverr_404();
        return Ok(Err(QuoteError::NotFound));
    };

    let page = DepositQuotePageData {
        form: DepositQuoteFormData {
            id: quote.id,
            quote_date: quote.quote_date,
            institution_id: quote.institution_id,
            term: quote.term,
            term_sort: quote.term_sort,
            currency: quote.currency,
            bid_rate: quote
                .bid_rate
                .map(|d| d.to_string()),
            ask_rate: quote
                .ask_rate
                .map(|d| d.to_string()),
            note: quote.note,
        },
    };
    Ok(Ok(page))
}

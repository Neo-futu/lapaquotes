use super::QuoteError;
use crate::form::FormFooter;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

#[component]
pub fn BondQuoteDeleteForm(id: String) -> impl IntoView {
    let action = ServerAction::<BondQuoteDelete>::new();
    let pending = action.pending();
    let value = action.value();

    Effect::new(move |_| {
        let v = value.get();
        if let Some(v) = v {
            let result = v.map_err(|_| QuoteError::ServerError).flatten();
            if let Ok(_) = result {
                tracing::info!("navigate bond quote delete ok");
                let navigate = use_navigate();
                navigate(&"/quote/bond", Default::default());
            }
        }
    });

    view! {
        <fieldset prop:disabled=move || pending()>
            <legend>Danger zone. Delete this bond quote.</legend>
            <ActionForm action=action>
                <input type="hidden" name="id" value=id.clone() />
                <p>"This cannot be undone."</p>
                <FormFooter action=action submit_text="Delete bond quote" />
            </ActionForm>
        </fieldset>
    }
}

#[server(BondQuoteDelete, "/api")]
pub async fn bond_quote_delete(id: String) -> Result<Result<(), QuoteError>, ServerFnError> {
    let db = crate::server::db::use_db().await?;
    let _deleted = cornucopia::queries::admin_quote::bond_delete()
        .bind(&db, &id)
        .await
        .map_err(|e| lib::emsg(e, "Bond quote delete"))?;
    Ok(Ok(()))
}

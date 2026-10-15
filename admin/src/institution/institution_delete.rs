use super::InstitutionError;
use crate::form::FormFooter;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

#[component]
pub fn InstitutionDeleteForm(id: String, name: String) -> impl IntoView {
    let action = ServerAction::<InstitutionDelete>::new();
    let pending = action.pending();
    let value = action.value();

    Effect::new(move |_| {
        let v = value.get();
        if let Some(v) = v {
            let institution_result = v.map_err(|_| InstitutionError::ServerError).flatten();
            if let Ok(_institution_result) = institution_result {
                tracing::info!("navigate institution_result ok");
                let navigate = use_navigate();
                let to = "/institution";
                navigate(&to, Default::default());
            }
        }
    });

    let (input_name, set_input_name) = signal::<String>("".to_string());
    let disabled = Memo::new(move |_| input_name() != name);
    view! {
        <fieldset prop:disabled=move || pending()>
            <legend>"Danger zone. Delete institution and all its quotes."</legend>
            <ActionForm action=action>
                <input type="hidden" name="id" value=id.clone() />
                <label>
                    <div>Name</div>
                    <input
                        value=input_name
                        prop:value=input_name
                        on:input=move |ev| {
                            set_input_name(event_target_value(&ev));
                        }
                    />
                </label>
                <FormFooter action=action submit_text="Delete institution" disabled />
            </ActionForm>
        </fieldset>
    }
}

#[server(InstitutionDelete, "/api")]
pub async fn institution_delete(id: String) -> Result<Result<(), InstitutionError>, ServerFnError> {
    let db = crate::server::db::use_db().await?;
    // DepositQuote / BondQuote rows cascade via FK.
    let _deleted = cornucopia::queries::admin_institution::delete()
        .bind(&db, &id)
        .await
        .map_err(|e| lib::emsg(e, "Institution delete"))?;
    Ok(Ok(()))
}

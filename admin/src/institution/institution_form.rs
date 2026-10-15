use super::InstitutionError;
use crate::form::{Checkbox, FormFooter, Input};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InstitutionFormData {
    pub id: String,
    pub name: String,
    pub short_name: Option<String>,
    pub email: Option<String>,
    pub is_active: bool,
}

#[component]
pub fn InstitutionForm(form: InstitutionFormData) -> impl IntoView {
    let institution_update = ServerAction::<InstitutionUpdate>::new();
    let pending = institution_update.pending();

    let short_name = form.short_name.clone().unwrap_or_default();
    let email = form.email.clone().unwrap_or_default();

    view! {
        <ActionForm action=institution_update>
            <fieldset prop:disabled=move || pending()>
                <legend>Data</legend>
                <input type="hidden" name="id" value=form.id />
                <div class="Grid-fluid-2">
                    <Input name="name" label="Name" value=form.name.clone() />
                    <Input name="short_name" label="Short name" value=short_name />
                    <Input name="email" label="Email (quotes source)" value=email />
                    <Checkbox
                        name="is_active"
                        label="Active"
                        checked=form.is_active
                    />
                </div>
                <FormFooter action=institution_update submit_text="Update institution" />
            </fieldset>
        </ActionForm>
    }
}
#[server(InstitutionUpdate, "/api")]
pub async fn institution_update(
    id: String,
    name: String,
    short_name: String,
    email: String,
    is_active: bool,
) -> Result<Result<(), InstitutionError>, ServerFnError> {
    let db = crate::server::db::use_db().await?;

    let trim_opt = |s: String| {
        let t = s.trim();
        if t.is_empty() {
            None
        } else {
            Some(t.to_string())
        }
    };
    let short_name = trim_opt(short_name);
    let email = trim_opt(email);
    let name = name.trim().to_string();

    // Uniqueness check (name is unique, excluding this row).
    let existing = cornucopia::queries::admin_institution::find_by_name()
        .bind(&db, &name)
        .opt()
        .await
        .map_err(|e| lib::emsg(e, "Institution find by name"))?;
    if let Some(row) = existing {
        if row.id != id {
            return Ok(Err(InstitutionError::CreateNameExists));
        }
    }

    cornucopia::queries::admin_institution::update()
        .bind(&db, &name, &short_name, &email, &is_active, &id)
        .opt()
        .await
        .map_err(|e| lib::emsg(e, "Institution update"))?;
    return Ok(Ok(()));
}

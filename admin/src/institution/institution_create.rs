use crate::form::{Checkbox, FormFooter, Input};
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_navigate;

use super::InstitutionError;

#[component]
pub fn InstitutionNew() -> impl IntoView {
    let institution_create = ServerAction::<InstitutionCreate>::new();
    let value = institution_create.value();
    let pending = institution_create.pending();
    Effect::new(move |_| {
        let v = value.get();
        if let Some(v) = v {
            let create_result = v.map_err(|_| InstitutionError::ServerError).flatten();
            if let Ok(id) = create_result {
                tracing::info!("navigate institution create_result ok");
                let navigate = use_navigate();
                let to = format!("/institution/{}", id);
                navigate(&to, Default::default());
            }
        }
    });
    view! {
        <Title text="New institution" />
        <h1>"New institution"</h1>
        <ActionForm action=institution_create>
            <fieldset prop:disabled=move || pending()>
                <legend>Data</legend>
                <div class="Grid-fluid-2">
                    <Input name="name" label="Name" />
                    <Input name="short_name" label="Short name" />
                    <Input name="email" label="Email (quotes source)" />
                    <Checkbox name="is_active" label="Active" checked=true />
                </div>
                <FormFooter action=institution_create submit_text="Create institution" />
            </fieldset>
        </ActionForm>
    }
    .into_any()
}

#[server(InstitutionCreate, "/api")]
pub async fn institution_create(
    name: String,
    short_name: String,
    email: String,
    is_active: bool,
) -> Result<Result<String, InstitutionError>, ServerFnError> {
    let db = crate::server::db::use_db().await?;
    let id = cuid2::create_id();

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

    // Uniqueness check.
    let existing = cornucopia::queries::admin_institution::find_by_name()
        .bind(&db, &name)
        .opt()
        .await
        .map_err(|e| lib::emsg(e, "Institution find by name"))?;
    if existing.is_some() {
        return Ok(Err(InstitutionError::CreateNameExists));
    }

    let _created_at = cornucopia::queries::admin_institution::create()
        .bind(&db, &id, &name, &short_name, &email, &is_active)
        .one()
        .await
        .map_err(|e| lib::emsg(e, "Institution create"))?;
    return Ok(Ok(id));
}

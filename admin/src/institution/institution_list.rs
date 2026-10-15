use chrono::{DateTime, FixedOffset};
use leptos::{either::Either, prelude::*};
use leptos_meta::Title;
use leptos_router::components::A;
use serde::{Deserialize, Serialize};

use crate::util::{AlertDanger, Loading, datetime_to_strings};

#[component]
pub fn InstitutionList() -> impl IntoView {
    let institutions = Resource::new_blocking(|| (), move |_| get_institutions());

    view! {
        <Title text="Institutions" />
        <h1>
            <span>"Institutions"</span>
            <span>" "</span>
            <A href="/institution/new">"+1"</A>
        </h1>

        <Suspense fallback=move || {
            view! { <Loading /> }
        }>
            {move || Suspend::new(async move {
                match institutions.await {
                    Err(e) => Either::Left(view! { <AlertDanger text=e.to_string() /> }),
                    Ok(institutions) => Either::Right(view! { <InstitutionListItems institutions /> }),
                }
            })}
        </Suspense>
    }
    .into_any()
}

#[component]
pub fn InstitutionListItems(institutions: Vec<InstitutionListItem>) -> impl IntoView {
    if institutions.is_empty() {
        Either::Left(view! { <p>"0 institutions"</p> })
    } else {
        Either::Right(view! {
            <ul class="Card Listing">
                {institutions
                    .into_iter()
                    .map(|institution| {
                        view! { <InstitutionListItem institution /> }
                    })
                    .collect_view()}

            </ul>
        })
    }
}

#[component]
pub fn InstitutionListItem(institution: InstitutionListItem) -> impl IntoView {
    let created = datetime_to_strings(institution.created_at);
    let active = if institution.is_active { "" } else { " (inactive)" };
    view! {
        <li>
            <div>{created.local}</div>
            <A href=format!("/institution/{}", institution.id)>{institution.name}</A>
            <div>
                {institution.short_name}
                {active}
            </div>
        </li>
    }
}

#[server(GetInstitutionsList, "/api")]
async fn get_institutions() -> Result<Vec<InstitutionListItem>, ServerFnError> {
    let db = crate::server::db::use_db().await?;
    let institutions = cornucopia::queries::admin_institution::list()
        .bind(&db)
        .all()
        .await
        .map_err(|e| lib::emsg(e, "Institutions list"))?;
    let institutions: Vec<InstitutionListItem> = institutions
        .into_iter()
        .map(|data| InstitutionListItem {
            id: data.id,
            created_at: data.created_at.and_utc().fixed_offset(),
            name: data.name,
            short_name: data.short_name,
            is_active: data.is_active,
        })
        .collect();
    Ok(institutions)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InstitutionListItem {
    pub id: String,
    pub created_at: DateTime<FixedOffset>,
    pub name: String,
    pub short_name: Option<String>,
    pub is_active: bool,
}

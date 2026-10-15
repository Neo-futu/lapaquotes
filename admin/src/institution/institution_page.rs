use leptos::{either::EitherOf3, prelude::*};
use leptos_meta::Title;
use leptos_router::{hooks::use_params, params::Params};
use serde::{Deserialize, Serialize};

use super::{InstitutionFormData, InstitutionHeaderData};
use crate::{
    institution::{InstitutionDeleteForm, InstitutionError, InstitutionForm, InstitutionHeader},
    util::{AlertDanger, Loading},
};

#[derive(Params, Clone, Debug, PartialEq, Eq)]
pub struct InstitutionParams {
    id: String,
}
#[derive(Default, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InstitutionPageData {
    pub header: InstitutionHeaderData,
    pub form: InstitutionFormData,
}

#[component]
pub fn InstitutionPage() -> impl IntoView {
    let params = use_params::<InstitutionParams>();
    let id = Memo::new(move |prev: Option<&Result<String, InstitutionError>>| {
        params.with(|q| {
            // Memo to fix Err(MissingParam("id")) when navigating away from page inside <Outlet />
            match q {
                Ok(q) => Ok(q.id.clone()),
                Err(_) => {
                    if let Some(Ok(prev)) = prev {
                        Ok(prev.to_owned())
                    } else {
                        Err(InstitutionError::InvalidId)
                    }
                }
            }
        })
    });

    let institution = Resource::new_blocking(
        move || id(),
        move |id| async move {
            match id {
                Err(e) => Ok(Err(e)),
                Ok(id) => get_institution(id).await,
            }
        },
    );

    view! {
        <Suspense fallback=move || {
            view! { <Loading /> }
        }>
            {move || Suspend::new(async move {
                match institution.await {
                    Ok(Ok(institution)) => {
                        EitherOf3::A(view! { <InstitutionPageView institution=institution /> })
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
pub fn InstitutionPageView(institution: InstitutionPageData) -> impl IntoView {
    view! {
        <Title text=format!("Institution: {}", institution.form.name) />
        <section class="PostPage">
            <InstitutionHeader institution=institution.header.clone() />
            <InstitutionForm form=institution.form.clone() />
            <InstitutionDeleteForm id=institution.header.id.clone() name=institution.form.name.clone() />
        </section>
    }
}

#[server(GetInstitution, "/api")]
pub async fn get_institution(
    id: String,
) -> Result<Result<InstitutionPageData, InstitutionError>, ServerFnError> {
    let db = crate::server::db::use_db().await?;
    let institution = cornucopia::queries::admin_institution::page()
        .bind(&db, &id)
        .opt()
        .await
        .map_err(|e| lib::emsg(e, "Institution find"))?;

    let Some(institution) = institution else {
        crate::server::serverr_404();
        return Ok(Err(InstitutionError::NotFound));
    };

    let institution_page = InstitutionPageData {
        header: InstitutionHeaderData {
            id: institution.id.clone(),
            created_at: institution.created_at.and_utc().fixed_offset(),
        },
        form: InstitutionFormData {
            id: institution.id,
            name: institution.name,
            short_name: institution.short_name,
            email: institution.email,
            is_active: institution.is_active,
        },
    };
    Ok(Ok(institution_page))
}

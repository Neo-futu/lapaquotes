use leptos::{either::Either, prelude::*};
use serde::{Deserialize, Serialize};

use crate::util::AlertDanger;

/// A `<select>` of institutions for use in quote forms (field name
/// `institution_id`). When `selected` is provided (edit page) it pre-selects
/// that institution.
#[component]
pub fn InstitutionSelect(#[prop(default = String::new())] selected: String) -> impl IntoView {
    let institution_resource = Resource::new_blocking(|| (), |_| get_institutions_select());
    view! {
        <Suspense>
            <label>
                <div>Institution</div>
                <select name="institution_id" autocomplete="off" required>
                    {move || {
                        let selected = selected.clone();
                        Suspend::new(async move {
                            match institution_resource.await {
                                Ok(institutions) => {
                                    Either::Left(
                                        institutions
                                            .into_iter()
                                            .map(|institution| {
                                                let is_selected = institution.id == selected;
                                                view! {
                                                    <option value=institution.id.clone() selected=is_selected>
                                                        {institution.name}
                                                    </option>
                                                }
                                            })
                                            .collect_view(),
                                    )
                                }
                                Err(e) => Either::Right(view! { <AlertDanger text=e.to_string() /> }),
                            }
                        })
                    }}
                </select>
            </label>
        </Suspense>
    }
}

#[derive(Default, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstitutionSelectData {
    pub id: String,
    pub name: String,
}
#[server(GetInstitutionSelect, "/api")]
async fn get_institutions_select() -> Result<Vec<InstitutionSelectData>, ServerFnError> {
    let db = crate::server::db::use_db().await?;
    let institutions = cornucopia::queries::admin_institution::list()
        .bind(&db)
        .map(|data| InstitutionSelectData {
            id: data.id.into(),
            name: data.name.into(),
        })
        .all()
        .await
        .map_err(|e| lib::emsg(e, "Institution select"))?;
    Ok(institutions)
}

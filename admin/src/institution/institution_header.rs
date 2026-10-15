use chrono::{DateTime, FixedOffset};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::util::datetime_to_strings;

#[derive(Default, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstitutionHeaderData {
    pub id: String,
    pub created_at: DateTime<FixedOffset>,
}
#[component]
pub fn InstitutionHeader(institution: InstitutionHeaderData) -> impl IntoView {
    let created = datetime_to_strings(institution.created_at);
    view! {
        <header>
            <div>
                <h1>Institution</h1>
            </div>
            <dl>
                <dt>ID:</dt>
                <dd>{institution.id}</dd>
                <br />
                <dt>Created at <small>(UTC):</small></dt>
                <dd>{created.utc}</dd>
            </dl>
        </header>
    }
}

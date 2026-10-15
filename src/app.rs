use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::{A, Router};

use crate::{routes::FrontRoutes, settings::SettingsCx};

#[component]
pub fn App(settings: SettingsCx) -> impl IntoView {
    provide_meta_context();
    provide_context(settings.clone());
    let formatter = |text| format!("{text} - Balance AM");
    view! {
        <Title formatter />
        <Router>
            <header>
                <section>
                    <div class="Brand">
                        <A href="/" exact=true>
                            <img src="/balance_logo.png" alt="Balance Asset Management" />
                        </A>
                    </div>
                    <A href="/" exact=true>
                        Quotes
                    </A>
                    <A href="/chat">Chat</A>
                </section>
            </header>
            <main>
                <FrontRoutes />
            </main>
        </Router>
    }
}

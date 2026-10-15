use leptos::prelude::*;
use leptos_router::{
    ParamSegment, StaticSegment,
    components::{Route, Router, Routes},
};

use crate::{
    chat::ChatPage, post::post_page::PostPage, product_page::ProductPage, quotes::QuotesPage,
};

#[component]
pub fn FrontRoutes() -> impl IntoView {
    view! {
        <Routes fallback=|| "Page not found">
            <Route path=StaticSegment("") view=QuotesPage />
            <Route path=StaticSegment("chat") view=ChatPage />
            <Route path=(StaticSegment("product"), ParamSegment("slug")) view=ProductPage />
            <Route path=(ParamSegment("category_slug"), ParamSegment("slug")) view=PostPage />
        </Routes>
    }
}

#[component]
pub fn GenerateRouteList() -> impl IntoView {
    leptos_meta::provide_meta_context();
    view! {
        <Router>
            <FrontRoutes />
        </Router>
    }
}

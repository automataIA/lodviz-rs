//! # web_dashboard
//!
//! Main dashboard application for lodviz-rs, built with Leptos.
//!
//! This crate contains the UI layout, routing, and data loading logic
//! to showcase the components provided by `lodviz_components` and `lodviz_core`.

use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{components::*, path};

/// Base URL path for asset fetches. Empty in dev, "/lodviz-rs" on GitHub Pages.
pub const BASE_PATH: &str = if cfg!(feature = "gh-pages") {
    "/lodviz-rs"
} else {
    ""
};
use lodviz_components::components::layout::global_mouse::provide_global_mouse;

// Core domain logic (public API)
// Core domain logic is now in lodviz_core crate
// pub mod algorithms;
// pub mod core;
/// Theme configuration and interactive editor for the dashboard
pub mod theme;

// Data loading
mod data;

// UI components (internal)
mod components;
mod pages;

// Top-Level pages
use crate::pages::home::Home;

/// An app router which renders the homepage and handles 404's
#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    // Register ONE global mousemove + mouseup listener shared by all DraggableCards.
    // Must be called before any DraggableCard is mounted.
    provide_global_mouse();

    // Global theme signal: "light" | "dark"
    let theme: RwSignal<&'static str> = RwSignal::new("light");
    provide_context(theme);

    view! {
        <Html attr:lang="en" attr:dir="ltr" attr:data-theme=move || theme.get() />

        // sets the document title
        <Title text="lodviz-rs — Interactive Visualization" />

        // injects metadata in the <head> of the page
        <Meta charset="UTF-8" />
        <Meta name="viewport" content="width=device-width, initial-scale=1.0" />

        <Router base=if cfg!(feature = "gh-pages") { "/lodviz-rs" } else { "" }>
            <Routes fallback=|| view! { <div>"404 — Not Found"</div> }>
                <Route path=path!("/") view=Home />
            </Routes>
        </Router>
    }
}

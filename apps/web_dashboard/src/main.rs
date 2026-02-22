//! # web_dashboard bin
//!
//! Entry point for the Leptos web application.

use leptos::prelude::*;
use web_dashboard::App;

fn main() {
    // set up logging
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();

    mount_to_body(|| {
        view! { <App /> }
    })
}

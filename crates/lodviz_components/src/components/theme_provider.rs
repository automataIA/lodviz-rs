/// ThemeProvider component for reactive theme context
///
/// Provides a `Signal<ChartTheme>` via Leptos context that all chart
/// components can consume. Supports automatic dark mode detection.
use leptos::prelude::*;
use lodviz_core::core::theme::ChartTheme;

/// Detect browser `prefers-color-scheme: dark` media query
///
/// Returns a reactive signal that updates when the user changes their OS theme.
pub fn use_prefers_dark() -> Signal<bool> {
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::closure::Closure;
        use wasm_bindgen::JsCast;

        let (is_dark, set_is_dark) = signal(false);

        // Initial check + listen for changes
        if let Some(Ok(Some(mql))) =
            web_sys::window().map(|w| w.match_media("(prefers-color-scheme: dark)"))
        {
            set_is_dark.set(mql.matches());

            let closure = Closure::<dyn Fn(web_sys::MediaQueryListEvent)>::new(
                move |ev: web_sys::MediaQueryListEvent| {
                    set_is_dark.set(ev.matches());
                },
            );

            let _ =
                mql.add_event_listener_with_callback("change", closure.as_ref().unchecked_ref());

            // Leak the closure to keep it alive (it's a global listener)
            closure.forget();
        }

        is_dark.into()
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        false.into()
    }
}

/// ThemeProvider component
///
/// Wraps children and provides a reactive `Signal<ChartTheme>` via context.
/// Charts read the theme from context if their own `config.theme` is `None`.
///
/// If `auto_dark_mode` is true (default), automatically switches between
/// `light_theme` and `dark_theme` based on the browser's `prefers-color-scheme`.
#[component]
pub fn ThemeProvider(
    /// Theme to use in light mode (default: `ChartTheme::default()`)
    #[prop(default = ChartTheme::default())]
    light_theme: ChartTheme,
    /// Theme to use in dark mode (default: `ChartTheme::dark()`)
    #[prop(default = ChartTheme::dark())]
    dark_theme: ChartTheme,
    /// Enable automatic dark mode detection
    #[prop(default = true)]
    auto_dark_mode: bool,
    /// Optional manual override — if Some, ignores auto detection
    #[prop(optional, into)]
    theme_override: Option<Signal<ChartTheme>>,
    /// Children
    children: Children,
) -> impl IntoView {
    let prefers_dark = use_prefers_dark();

    let theme_signal: Signal<ChartTheme> = if let Some(override_sig) = theme_override {
        override_sig
    } else {
        Signal::derive(move || {
            if auto_dark_mode && prefers_dark.get() {
                dark_theme.clone()
            } else {
                light_theme.clone()
            }
        })
    };

    provide_context(theme_signal);

    children()
}

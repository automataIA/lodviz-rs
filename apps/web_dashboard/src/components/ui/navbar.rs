use leptos::prelude::*;

/// Top navigation bar with theme toggle (light / dark).
///
/// Reads the `RwSignal<&'static str>` theme from context (provided by `App`).
/// Uses the DaisyUI `theme-controller` pattern for the toggle button.
#[component]
pub fn Navbar() -> impl IntoView {
    let theme = use_context::<RwSignal<&'static str>>().expect("theme RwSignal must be in context");

    let is_dark = Signal::derive(move || theme.get() == "dark");

    view! {
        <div class="navbar bg-base-100 border-b border-base-300 px-4 min-h-14 h-14 z-50 shrink-0 flex items-center justify-between">
            // Brand
            <div class="flex-none">
                <span class="text-xl font-bold text-primary">"lodviz-rs"</span>
                <span class="ml-2 text-xs opacity-40 font-mono">"v0.2 · SVG + WASM"</span>
            </div>

            // Instructions (Centered)
            <div class="flex-1 text-center text-xs opacity-60">
                "Drag cards to move • Resize from corner • Snap to grid"
            </div>

            // Theme toggle — DaisyUI swap + theme-controller pattern
            <div class="flex-none">
                <label class="swap swap-rotate">
                    <input
                        type="checkbox"
                        class="theme-controller"
                        value="dark"
                        prop:checked=is_dark
                        on:change=move |_| {
                            theme
                                .update(|t| {
                                    *t = if *t == "dark" { "light" } else { "dark" };
                                });
                        }
                    />

                    // Sun icon — visible in light mode (swap-off)
                    <svg
                        class="swap-off h-8 w-8 fill-current"
                        xmlns="http://www.w3.org/2000/svg"
                        viewBox="0 0 24 24"
                    >
                        <path d="M5.64,17l-.71.71a1,1,0,0,0,0,1.41,1,1,0,0,0,1.41,0l.71-.71A1,1,0,0,0,5.64,17ZM5,12a1,1,0,0,0-1-1H3a1,1,0,0,0,0,2H4A1,1,0,0,0,5,12Zm7-7a1,1,0,0,0,1-1V3a1,1,0,0,0-2,0V4A1,1,0,0,0,12,5ZM5.64,7.05a1,1,0,0,0,.7.29,1,1,0,0,0,.71-.29,1,1,0,0,0,0-1.41l-.71-.71A1,1,0,0,0,4.93,6.34Zm12,.29a1,1,0,0,0,.7-.29l.71-.71a1,1,0,1,0-1.41-1.41L17,5.64a1,1,0,0,0,0,1.41A1,1,0,0,0,17.66,7.34ZM21,11H20a1,1,0,0,0,0,2h1a1,1,0,0,0,0-2Zm-9,8a1,1,0,0,0-1,1v1a1,1,0,0,0,2,0V20A1,1,0,0,0,12,19ZM18.36,17A1,1,0,0,0,17,18.36l.71.71a1,1,0,0,0,1.41,0,1,1,0,0,0,0-1.41ZM12,6.5A5.5,5.5,0,1,0,17.5,12,5.51,5.51,0,0,0,12,6.5Zm0,9A3.5,3.5,0,1,1,15.5,12,3.5,3.5,0,0,1,12,15.5Z" />
                    </svg>

                    // Moon icon — visible in dark mode (swap-on)
                    <svg
                        class="swap-on h-8 w-8 fill-current"
                        xmlns="http://www.w3.org/2000/svg"
                        viewBox="0 0 24 24"
                    >
                        <path d="M21.64,13a1,1,0,0,0-1.05-.14,8.05,8.05,0,0,1-3.37.73A8.15,8.15,0,0,1,9.08,5.49a8.59,8.59,0,0,1,.25-2A1,1,0,0,0,8,2.36,10.14,10.14,0,1,0,22,14.05,1,1,0,0,0,21.64,13Zm-9.5,6.69A8.14,8.14,0,0,1,7.08,5.22v.27A10.15,10.15,0,0,0,17.22,15.63a9.79,9.79,0,0,0,2.1-.22A8.11,8.11,0,0,1,12.14,19.73Z" />
                    </svg>
                </label>
            </div>
        </div>
    }
}

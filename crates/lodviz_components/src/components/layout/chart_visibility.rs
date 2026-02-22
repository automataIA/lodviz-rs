/// Lazy-rendering wrapper that mounts children only when the container
/// enters the browser viewport, using `IntersectionObserver`.
///
/// Wrapping a heavy chart component with `ChartVisibility` avoids paying the
/// SVG layout and paint cost for cards that are scrolled off-screen.
///
/// ## Design notes
///
/// * `children: ChildrenFn` — `<Show>` requires a callable-multiple-times
///   children type.  In practice the closure is invoked **at most once** because
///   `has_been_visible` only ever transitions `false → true`.
/// * The `Closure` and `IntersectionObserver` are `.forget()`/`mem::forget()`-ed
///   instead of cleaned up via `on_cleanup`.  `on_cleanup` requires `Send + Sync`,
///   which raw-pointer JS values don't satisfy in the reactive-graph crate.
///   Since `DraggableCard`s in the dashboard are permanent (never unmounted),
///   this is safe: the browser's GC keeps each observer alive through its active
///   DOM observations, and the callbacks are simply never reclaimed.
/// * Placed **inside** a `DraggableCard`, so the parent's
///   `provide_context::<Signal<CardTransform>>` is already in scope when the
///   chart component is finally mounted.
use leptos::html;
use leptos::prelude::*;
use leptos::wasm_bindgen::closure::Closure;
use leptos::wasm_bindgen::JsCast;
use leptos::web_sys;
use web_sys::js_sys;

/// Conditionally mounts `children` only after the wrapper `<div>` intersects
/// the viewport for the first time.
///
/// Once visible, the children remain mounted permanently (no unmount on scroll-away).
#[component]
pub fn ChartVisibility(children: ChildrenFn) -> impl IntoView {
    // `has_been_visible` latches to `true` on first intersection and never resets.
    // This ensures the chart is mounted exactly once and then kept alive.
    let has_been_visible = RwSignal::new(false);
    let container_ref = NodeRef::<html::Div>::new();

    // Register the IntersectionObserver once the wrapper div is in the DOM.
    Effect::new(move |_| {
        let Some(el) = container_ref.get() else {
            return;
        };

        // Callback: set `has_been_visible = true` on first intersection.
        // We deliberately capture `has_been_visible` (Copy) and check
        // `get_untracked()` to avoid creating a reactive dependency here.
        let on_intersect =
            Closure::<dyn FnMut(js_sys::Array, web_sys::IntersectionObserver)>::wrap(Box::new(
                move |entries: js_sys::Array, _observer: web_sys::IntersectionObserver| {
                    if has_been_visible.get_untracked() {
                        return; // already latched, nothing to do
                    }
                    if let Ok(entry) = entries
                        .get(0)
                        .dyn_into::<web_sys::IntersectionObserverEntry>()
                    {
                        if entry.is_intersecting() {
                            has_been_visible.set(true);
                        }
                    }
                },
            ));

        let observer = web_sys::IntersectionObserver::new(on_intersect.as_ref().unchecked_ref())
            .expect("IntersectionObserver::new failed");

        observer.observe(&el);

        // `on_cleanup` requires `Send + Sync`, which JsValue-based types don't
        // satisfy.  Since cards are permanent, we forget both handles instead.
        // The browser keeps the observer alive through its active DOM observation.
        on_intersect.forget();
        std::mem::forget(observer);
    });

    view! {
        <div node_ref=container_ref style="width: 100%; height: 100%;">
            <Show when=move || has_been_visible.get()>{children()}</Show>
        </div>
    }
}

/// Hook for tracking container element dimensions using ResizeObserver
use leptos::html::Div;
use leptos::prelude::*;
use std::mem;
use wasm_bindgen::prelude::*;
use web_sys::js_sys;

/// Returns (width_signal, height_signal, container_ref) for tracking element dimensions.
///
/// The returned `NodeRef<Div>` must be attached to the container `<div>` element you want to measure.
/// The width and height signals will update whenever the element is resized.
///
/// # Example
///
/// ```rust,ignore
/// let (width, height, container_ref) = use_container_size();
///
/// view! {
///     <div node_ref=container_ref style="width: 100%; height: 100%;">
///         <svg viewBox=move || format!("0 0 {} {}", width.get(), height.get())>
///             // ... chart content
///         </svg>
///     </div>
/// }
/// ```
pub fn use_container_size() -> (Signal<f64>, Signal<f64>, NodeRef<Div>) {
    let container_ref = NodeRef::<Div>::new();
    let (width, set_width) = signal(0.0_f64);
    let (height, set_height) = signal(0.0_f64);

    Effect::new(move |_| {
        let Some(element) = container_ref.get() else {
            return;
        };

        // Initial measurement
        let rect = element.get_bounding_client_rect();
        set_width.set(rect.width());
        set_height.set(rect.height());

        // Set up ResizeObserver for dynamic updates
        let set_width_clone = set_width;
        let set_height_clone = set_height;

        let closure = Closure::wrap(Box::new(move |entries: js_sys::Array| {
            if let Ok(entry) = entries.get(0).dyn_into::<web_sys::ResizeObserverEntry>() {
                let content_rect = entry.content_rect();
                set_width_clone.set(content_rect.width());
                set_height_clone.set(content_rect.height());
            }
        }) as Box<dyn FnMut(js_sys::Array)>);

        let observer = web_sys::ResizeObserver::new(closure.as_ref().unchecked_ref())
            .expect("Failed to create ResizeObserver");

        observer.observe(&element);

        // Leak closure and observer (component lifetime)
        // We can't use on_cleanup here because Closure is !Send
        mem::forget(closure);
        mem::forget(observer);
    });

    (width.into(), height.into(), container_ref)
}

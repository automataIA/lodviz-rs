/// Centralized global mouse state to avoid per-card DOM listener leaks.
///
/// Instead of each DraggableCard registering its own `mousemove` + `mouseup`
/// listeners on `document` (which accumulate via `Closure::forget()`), we
/// register a single pair on `window` here and expose reactive signals.
use leptos::prelude::*;
use leptos::wasm_bindgen::closure::Closure;
use leptos::wasm_bindgen::JsCast;
use leptos::web_sys;
use std::sync::atomic::{AtomicU64, Ordering};

/// Monotonic counter for assigning unique `u64` IDs to `DraggableCard` instances.
/// `u64` is `Copy`, so closures that capture it remain `Copy`.
static CARD_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Allocate the next unique card ID.
pub fn next_card_id() -> u64 {
    CARD_ID_COUNTER.fetch_add(1, Ordering::Relaxed)
}

/// Shared reactive mouse state, provided at the App root.
///
/// Uses `u64` IDs so that all fields are `RwSignal<T>` with `T: Copy`,
/// keeping the struct `Copy` and making handler closures `Copy` too.
#[derive(Clone, Copy)]
pub struct GlobalMouseState {
    /// Current mouse position in client coordinates (client_x, client_y).
    pub position: RwSignal<(f64, f64)>,
    /// Numeric ID of the card that is currently being dragged or resized.
    /// `None` means no card is active. `u64` is `Copy`.
    pub active_card: RwSignal<Option<u64>>,
}

impl GlobalMouseState {
    fn new() -> Self {
        Self {
            position: RwSignal::new((0.0, 0.0)),
            active_card: RwSignal::new(None),
        }
    }
}

/// Register ONE `mousemove` + ONE `mouseup` listener on `window`.
///
/// Call this once inside the root `App` component (or the layout wrapping all
/// `DraggableCard`s). Since `App` is never unmounted, the closures are safely
/// forgotten after registration — no listener leak occurs.
pub fn provide_global_mouse() {
    let state = GlobalMouseState::new();
    provide_context(state);

    let position = state.position;
    let active_card = state.active_card;

    let window = web_sys::window().expect("window");

    let on_move = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
        position.set((e.client_x() as f64, e.client_y() as f64));
    }) as Box<dyn FnMut(web_sys::MouseEvent)>);

    let on_up = Closure::wrap(Box::new(move |_: web_sys::MouseEvent| {
        active_card.set(None);
    }) as Box<dyn FnMut(web_sys::MouseEvent)>);

    let _ = window.add_event_listener_with_callback("mousemove", on_move.as_ref().unchecked_ref());
    let _ = window.add_event_listener_with_callback("mouseup", on_up.as_ref().unchecked_ref());

    // App is the root component and is never unmounted — safe to forget.
    on_move.forget();
    on_up.forget();
}

/// Retrieve the `GlobalMouseState` from context.
///
/// Panics if `provide_global_mouse()` was not called in an ancestor component.
pub fn use_global_mouse() -> GlobalMouseState {
    use_context::<GlobalMouseState>()
        .expect("GlobalMouseState not found — call provide_global_mouse() in App")
}

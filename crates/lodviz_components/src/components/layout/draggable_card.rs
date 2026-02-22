use super::card_registry::{register_card_transform, update_card_transform};
use super::global_mouse::{next_card_id, use_global_mouse};
use leptos::html::Div;
use leptos::prelude::*;
use leptos::web_sys::MouseEvent;
use lodviz_core::core::theme::ColorScheme;

/// Context key for providing the card ID to children
#[derive(Clone)]
pub struct CardId(pub String);

/// Position and size of a card
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct CardTransform {
    /// X position in pixels
    pub x: f64,
    /// Y position in pixels
    pub y: f64,
    /// Width in pixels
    pub width: f64,
    /// Height in pixels
    pub height: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ResizeHandle {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl Default for CardTransform {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 400.0,
            height: 300.0,
        }
    }
}

/// Draggable and resizable card component
///
/// Features:
/// - Drag to move (via header)
/// - Resize handles (corners and edges)
/// - Snap to grid
/// - Min/max constraints
#[component]
pub fn DraggableCard(
    /// Initial position and size
    #[prop(default = CardTransform::default())]
    initial_transform: CardTransform,

    /// Color scheme for the card border
    #[prop(optional)]
    color_scheme: Option<ColorScheme>,
    /// Snap to grid size (pixels)
    #[prop(default = 20)]
    snap_size: i32,
    /// Minimum width
    #[prop(default = 300.0)]
    #[allow(dead_code)]
    min_width: f64,
    /// Minimum height
    #[prop(default = 200.0)]
    #[allow(dead_code)]
    min_height: f64,
    /// Maximum width (optional)
    #[prop(optional)]
    #[allow(dead_code)]
    max_width: Option<f64>,
    /// Maximum height (optional)
    #[prop(optional)]
    #[allow(dead_code)]
    max_height: Option<f64>,
    /// Card content
    #[allow(dead_code)]
    children: Children,
) -> impl IntoView {
    // Unique numeric ID for this card — u64 is Copy, so closures remain Copy.
    let my_id: u64 = next_card_id();

    // String ID for the registry (separate from my_id)
    let card_id = my_id.to_string();

    // Card transform state
    let (transform, set_transform) = signal(initial_transform);

    // Register initial transform in global registry
    register_card_transform(&card_id, initial_transform);

    // Provide the card ID to children
    provide_context::<CardId>(CardId(card_id.clone()));

    // Provide the card transform as a reactive signal so child charts can read actual dimensions
    let transform_signal: Signal<CardTransform> = transform.into();
    provide_context::<Signal<CardTransform>>(transform_signal);

    // Update registry when transform changes
    Effect::new(move |_| {
        let t = transform.get();
        // Reuse my_id directly — u64 is Copy
        update_card_transform(&my_id.to_string(), t);
    });

    // Drag state
    let (is_dragging, set_is_dragging) = signal(false);
    let (drag_start, set_drag_start) = signal((0.0_f64, 0.0_f64));
    let (initial_pos, set_initial_pos) = signal((0.0_f64, 0.0_f64));

    // Resize state
    let (is_resizing, set_is_resizing) = signal(false);
    let (resize_handle, set_resize_handle) = signal(Option::<ResizeHandle>::None);
    let (resize_start, set_resize_start) = signal((0.0_f64, 0.0_f64));
    let (initial_transform_state, set_initial_transform_state) = signal(initial_transform);

    // Border hover state
    let (is_border_hovered, set_is_border_hovered) = signal(false);

    let card_ref = NodeRef::<Div>::new();

    // Snap to grid helper
    let snap_to_grid = move |value: f64| -> f64 {
        if snap_size > 0 {
            (value / snap_size as f64).round() * snap_size as f64
        } else {
            value
        }
    };

    // Constrain size helper
    let constrain_size = move |width: f64, height: f64| -> (f64, f64) {
        let w = width.max(min_width);
        let w = if let Some(max_w) = max_width {
            w.min(max_w)
        } else {
            w
        };
        let h = height.max(min_height);
        let h = if let Some(max_h) = max_height {
            h.min(max_h)
        } else {
            h
        };
        (w, h)
    };

    // Retrieve the single global mouse listener registered in App.
    // GlobalMouseState is Copy; my_id is u64 (Copy) → closures remain Copy.
    let global_mouse = use_global_mouse();

    // Start drag — set local state and mark this card as active.
    // All captures (signals + u64) are Copy → closure is Copy.
    let on_mouse_down_drag = move |e: MouseEvent| {
        e.prevent_default();
        set_is_dragging(true);
        set_drag_start((e.client_x() as f64, e.client_y() as f64));
        let t = transform.get();
        set_initial_pos((t.x, t.y));
        global_mouse.active_card.set(Some(my_id));
    };

    // Start resize — set local state and mark this card as active.
    let on_mouse_down_resize = move |e: MouseEvent, handle: ResizeHandle| {
        e.prevent_default();
        e.stop_propagation();
        set_is_resizing(true);
        set_resize_handle(Some(handle));
        set_resize_start((e.client_x() as f64, e.client_y() as f64));
        set_initial_transform_state(transform.get());
        global_mouse.active_card.set(Some(my_id));
    };

    // React to global mouse position: process drag/resize only when this card is active.
    Effect::new(move |_| {
        let (mx, my) = global_mouse.position.get(); // reactive dependency on position

        // Short-circuit if this card is not the active one.
        if global_mouse.active_card.get_untracked() != Some(my_id) {
            return;
        }

        if is_dragging.get_untracked() {
            let (start_x, start_y) = drag_start.get_untracked();
            let (init_x, init_y) = initial_pos.get_untracked();
            let dx = mx - start_x;
            let dy = my - start_y;

            let new_x = snap_to_grid(init_x + dx);
            let new_y = snap_to_grid(init_y + dy);

            set_transform.update(|t| {
                t.x = new_x;
                t.y = new_y;
            });
        } else if let Some(handle) = resize_handle.get_untracked() {
            let (start_x, start_y) = resize_start.get_untracked();
            let t = initial_transform_state.get_untracked();

            let dx = mx - start_x;
            let dy = my - start_y;

            let (mut new_x, mut new_y, mut new_w, mut new_h) = match handle {
                ResizeHandle::BottomRight => (t.x, t.y, t.width + dx, t.height + dy),
                ResizeHandle::BottomLeft => (t.x + dx, t.y, t.width - dx, t.height + dy),
                ResizeHandle::TopRight => (t.x, t.y + dy, t.width + dx, t.height - dy),
                ResizeHandle::TopLeft => (t.x + dx, t.y + dy, t.width - dx, t.height - dy),
            };

            // Snap to grid
            if snap_size > 0 {
                new_x = snap_to_grid(new_x);
                new_y = snap_to_grid(new_y);
                new_w = snap_to_grid(new_w);
                new_h = snap_to_grid(new_h);
            }

            // Constrain size
            let (final_w, final_h) = constrain_size(new_w, new_h);

            // Prevent position jump when hitting min size on left/top handles
            if handle == ResizeHandle::TopLeft || handle == ResizeHandle::BottomLeft {
                new_x = t.x + (t.width - final_w);
            }
            if handle == ResizeHandle::TopLeft || handle == ResizeHandle::TopRight {
                new_y = t.y + (t.height - final_h);
            }

            set_transform.update(|t| {
                t.x = new_x;
                t.y = new_y;
                t.width = final_w;
                t.height = final_h;
            });
        }
    });

    // Reset local drag/resize state when the global active_card is cleared (global mouseup).
    Effect::new(move |_| {
        if global_mouse.active_card.get().is_none() {
            set_is_dragging(false);
            set_is_resizing(false);
            set_resize_handle(None);
        }
    });

    view! {
        <div
            node_ref=card_ref
            class="draggable-card card absolute bg-base-100 shadow-xl flex flex-col overflow-visible z-10"
            class:border-4=move || is_border_hovered.get() || is_resizing.get() || is_dragging.get()
            class:border=move || {
                !(is_border_hovered.get() || is_resizing.get() || is_dragging.get())
            }
            style=move || {
                let t = transform.get();
                let border_color = color_scheme.map(|cs| cs.primary()).unwrap_or("#ddd");
                format!(
                    "left: {}px; top: {}px; width: {}px; height: {}px; border-color: {};",
                    t.x,
                    t.y,
                    t.width,
                    t.height,
                    border_color,
                )
            }
            on:mouseenter=move |_| set_is_border_hovered(true)
            on:mouseleave=move |_| set_is_border_hovered(false)
        >
            // Borders (invisible/overlay for dragging)
            <div
                style="position: absolute; top: 0; left: 0; right: 0; height: 10px; cursor: move; z-index: 10;"
                on:mousedown=on_mouse_down_drag
            />
            <div
                style="position: absolute; bottom: 0; left: 0; right: 0; height: 10px; cursor: move; z-index: 10;"
                on:mousedown=on_mouse_down_drag
            />
            <div
                style="position: absolute; top: 0; bottom: 0; left: 0; width: 10px; cursor: move; z-index: 10;"
                on:mousedown=on_mouse_down_drag
            />
            <div
                style="position: absolute; top: 0; bottom: 0; right: 0; width: 10px; cursor: move; z-index: 10;"
                on:mousedown=on_mouse_down_drag
            />
            // Header removed - drag via borders now

            // Content area
            <div class="card-body p-4" style="flex: 1; overflow: hidden; position: relative;">
                {children()}
            </div>

            // Resize handles
            // Bottom-Right
            <div
                class="resize-handle-br"
                on:mousedown=move |e| on_mouse_down_resize(e, ResizeHandle::BottomRight)
                style="position: absolute; bottom: -10px; right: -10px; width: 40px; height: 40px; \
                cursor: nwse-resize; z-index: 50;"
            />
            // Bottom-Left
            <div
                class="resize-handle-bl"
                on:mousedown=move |e| on_mouse_down_resize(e, ResizeHandle::BottomLeft)
                style="position: absolute; bottom: -10px; left: -10px; width: 40px; height: 40px; \
                cursor: nesw-resize; z-index: 50;"
            />
            // Top-Right
            <div
                class="resize-handle-tr"
                on:mousedown=move |e| on_mouse_down_resize(e, ResizeHandle::TopRight)
                style="position: absolute; top: -10px; right: -10px; width: 40px; height: 40px; \
                cursor: nesw-resize; z-index: 50;"
            />
            // Top-Left
            <div
                class="resize-handle-tl"
                on:mousedown=move |e| on_mouse_down_resize(e, ResizeHandle::TopLeft)
                style="position: absolute; top: -10px; left: -10px; width: 40px; height: 40px; \
                cursor: nwse-resize; z-index: 50;"
            />
        </div>
    }
}

/// Zoom and Pan interaction for charts
///
/// Provides a `ZoomTransform` signal that modifies chart scale domains based on user interactions.
///
/// # User Interactions
///
/// | Interaction | Behavior |
/// |-------------|----------|
/// | **Ctrl + drag** | Box zoom: draws a selection rectangle and zooms to the selected area |
/// | **Double click** | Reset zoom: restores the original unzoomed domain |
/// | **Mouse wheel** | Zoom in/out centered at cursor position (if `enable_zoom = true`) |
/// | **Mouse move** | Updates cursor position for tooltips (via `set_cursor` prop) |
///
/// # Example
///
/// ```rust,ignore
/// use lodviz_components::ZoomPan;
/// use leptos::prelude::*;
///
/// let (transform, set_transform) = signal(ZoomTransform::from_domain(0.0, 100.0, 0.0, 50.0));
/// let original = transform.read_only(); // Store original for reset
///
/// view! {
///     <ZoomPan
///         transform=set_transform
///         original=original
///         inner_width=Signal::derive(|| 800.0)
///         inner_height=Signal::derive(|| 400.0)
///     />
/// }
/// ```
use leptos::prelude::*;
use wasm_bindgen::JsCast;
// use web_sys::MouseEvent; // Unused import
/// Transform state for zoom and pan
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ZoomTransform {
    /// X domain start (after zoom/pan)
    pub x_min: f64,
    /// X domain end (after zoom/pan)
    pub x_max: f64,
    /// Y domain start (after zoom/pan)
    pub y_min: f64,
    /// Y domain end (after zoom/pan)
    pub y_max: f64,
}

impl ZoomTransform {
    /// Create from original data domain extents
    pub fn from_domain(x_min: f64, x_max: f64, y_min: f64, y_max: f64) -> Self {
        Self {
            x_min,
            x_max,
            y_min,
            y_max,
        }
    }

    /// Apply zoom centered at a normalized position (0..1)
    pub fn zoom(&self, factor: f64, center_x: f64, center_y: f64) -> Self {
        let x_range = self.x_max - self.x_min;
        let y_range = self.y_max - self.y_min;

        // Calculate the domain value at the center point (cursor position)
        // This value must remain at the same screen position (center_x, center_y) after scaling
        let domain_cx = self.x_min + center_x * x_range;
        let domain_cy = self.y_min + center_y * y_range;

        // Calculate new ranges based on the zoom factor
        // factor > 1.0 means zoom IN (smaller range)
        // factor < 1.0 means zoom OUT (larger range)
        let new_x_range = x_range / factor;
        let new_y_range = y_range / factor;

        // Calculate new min/max such that domain_cx is still at center_x relative position
        // new_x_min + center_x * new_x_range = domain_cx
        // => new_x_min = domain_cx - center_x * new_x_range
        let new_x_min = domain_cx - center_x * new_x_range;
        let new_x_max = new_x_min + new_x_range;

        let new_y_min = domain_cy - center_y * new_y_range;
        let new_y_max = new_y_min + new_y_range;

        Self {
            x_min: new_x_min,
            x_max: new_x_max,
            y_min: new_y_min,
            y_max: new_y_max,
        }
    }

    /// Apply box zoom based on selection rectangle (normalized coordinates 0..1)
    pub fn zoom_to_box(&self, x1: f64, y1: f64, x2: f64, y2: f64) -> Self {
        let x_range = self.x_max - self.x_min;
        let y_range = self.y_max - self.y_min;

        // Ensure proper ordering
        let start_x = x1.min(x2);
        let end_x = x1.max(x2);
        // SVG Y is top-down, but our domain logic might rely on min/max.
        // Usually charts map Y=0 to bottom, but SVG Y=0 is top.
        // Let's assume input y1/y2 are 0..1 from top-left.
        // If y_min is "bottom value" and y_max is "top value", we need to be careful.
        // However, ZoomTransform usually stores data domain.
        // y_min = value at bottom, y_max = value at top?
        // Let's assume ZoomTransform stores strict data min/max.
        // And standard mapping: y=0 (top) -> y_max, y=1 (bottom) -> y_min.
        // Then:
        // top_y_norm = start_y (smaller value 0..1) -> corresponds to HIGHER data value
        // bottom_y_norm = end_y (larger value 0..1) -> corresponds to LOWER data value

        // Let's verify standard linear scale behavior in this codebase.
        // y_scale: (y_min, y_max) -> (height, 0).
        // So y_min maps to height (bottom), y_max maps to 0 (top).

        let start_y_norm = y1.min(y2); // Closer to 0 (Top)
        let end_y_norm = y1.max(y2); // Closer to 1 (Bottom)

        // New y_max corresponds to start_y_norm (top of selection)
        // distance from top (0) is start_y_norm.
        // value = y_max - start_y_norm * y_range
        let new_y_max = self.y_max - start_y_norm * y_range;

        // New y_min corresponds to end_y_norm (bottom of selection)
        // value = y_max - end_y_norm * y_range
        let new_y_min = self.y_max - end_y_norm * y_range;

        // X logic (standard left-to-right)
        // x_min maps to 0, x_max maps to width.
        // value = x_min + norm * range
        let new_x_min = self.x_min + start_x * x_range;
        let new_x_max = self.x_min + end_x * x_range;

        Self {
            x_min: new_x_min,
            x_max: new_x_max,
            y_min: new_y_min,
            y_max: new_y_max,
        }
    }

    /// Apply pan by delta in domain units
    pub fn pan(&self, dx: f64, dy: f64) -> Self {
        Self {
            x_min: self.x_min + dx,
            x_max: self.x_max + dx,
            y_min: self.y_min + dy,
            y_max: self.y_max + dy,
        }
    }

    /// Reset to original domain
    pub fn reset(original: &ZoomTransform) -> Self {
        *original
    }
}

/// ZoomPan SVG overlay component
///
/// Renders a transparent rect that captures mouse events for zoom and pan interactions.
/// Updates a `RwSignal<ZoomTransform>` that charts should use to adjust their scales.
///
/// # Interaction Details
///
/// - **Ctrl + Mouse Down + Drag**: Starts a box selection. When mouse is released,
///   the chart zooms to the selected rectangle area. A semi-transparent blue rectangle
///   is shown during selection.
///
/// - **Double Click**: Immediately resets the zoom to the original domain provided
///   via the `original` prop.
///
/// - **Mouse Wheel**: Zooms in/out centered at the cursor position (enabled by default,
///   controlled by `enable_zoom` prop).
///
/// # Props
///
/// - `transform`: Read/write signal for the current zoom state
/// - `original`: Read-only signal containing the original unzoomed domain (for reset)
/// - `inner_width` / `inner_height`: Chart area dimensions in pixels
/// - `enable_zoom`: Enable/disable zoom interactions (default: true)
/// - `set_cursor`: Optional callback to propagate cursor position to tooltips
#[component]
pub fn ZoomPan(
    /// Current zoom transform (read/write)
    transform: RwSignal<ZoomTransform>,
    /// Original unzoomed domain (for reset)
    #[prop(into)]
    original: Signal<ZoomTransform>,
    /// Inner width of the chart area
    #[prop(into)]
    inner_width: Signal<f64>,
    /// Inner height of the chart area
    #[prop(into)]
    inner_height: Signal<f64>,
    /// Enable zoom on scroll
    #[prop(default = true)]
    enable_zoom: bool,
    /// Enable pan on drag
    #[prop(default = true)]
    _enable_pan: bool, // Deprecated/Unused, kept for API compat or future use
    /// Optional cursor position setter (for tooltips)
    #[prop(optional)]
    set_cursor: Option<WriteSignal<Option<(f64, f64)>>>,
) -> impl IntoView {
    let (selection_start, set_selection_start) = signal(None::<(f64, f64)>);
    let (selection_current, set_selection_current) = signal(None::<(f64, f64)>);
    let _is_selecting = Memo::new(move |_| selection_start.get().is_some());

    // Selection rectangle style
    let selection_rect = Memo::new(move |_| {
        let start = selection_start.get();
        let current = selection_current.get();

        match (start, current) {
            (Some((x1, y1)), Some((x2, y2))) => {
                let w = inner_width.get();
                let h = inner_height.get();

                let x = x1.min(x2) * w;
                let y = y1.min(y2) * h;
                let width = (x2 - x1).abs() * w;
                let height = (y2 - y1).abs() * h;

                Some((x, y, width, height))
            }
            _ => None,
        }
    });

    let on_mousedown = move |ev: web_sys::MouseEvent| {
        if !enable_zoom {
            return;
        }

        // Only start selection if Ctrl is pressed
        if !ev.ctrl_key() {
            return;
        }

        let rect = ev
            .target()
            .unwrap()
            .unchecked_into::<web_sys::Element>()
            .get_bounding_client_rect();
        let x = ev.client_x() as f64 - rect.left();
        let y = ev.client_y() as f64 - rect.top();
        // Use rendered CSS pixel dimensions (not SVG coordinate units)
        // to correctly normalize when the SVG is scaled via style="width:100%"
        let w = rect.width();
        let h = rect.height();

        let norm_x = (x / w).clamp(0.0, 1.0);
        let norm_y = (y / h).clamp(0.0, 1.0);

        set_selection_start.set(Some((norm_x, norm_y)));
        set_selection_current.set(Some((norm_x, norm_y)));

        // Also update cursor for external consumers
        if let Some(setter) = set_cursor {
            setter.set(Some((norm_x, norm_y)));
        }

        ev.prevent_default();
    };

    let on_mousemove = move |ev: web_sys::MouseEvent| {
        let rect = ev
            .target()
            .unwrap()
            .unchecked_into::<web_sys::Element>()
            .get_bounding_client_rect();
        let x = ev.client_x() as f64 - rect.left();
        let y = ev.client_y() as f64 - rect.top();
        // Use rendered CSS pixel dimensions (not SVG coordinate units)
        // to correctly normalize when the SVG is scaled via style="width:100%"
        let w = rect.width();
        let h = rect.height();

        let norm_x = (x / w).clamp(0.0, 1.0);
        let norm_y = (y / h).clamp(0.0, 1.0);

        if let Some(setter) = set_cursor {
            setter.set(Some((norm_x, norm_y)));
        }

        if selection_start.get().is_some() {
            set_selection_current.set(Some((norm_x, norm_y)));
        }
    };

    let on_mouseup = move |ev: web_sys::MouseEvent| {
        if let (Some((x1, y1)), Some((x2, y2))) = (selection_start.get(), selection_current.get()) {
            // Apply zoom if selection is big enough to be intentional
            if (x1 - x2).abs() > 0.01 && (y1 - y2).abs() > 0.01 {
                transform.update(|t| {
                    *t = t.zoom_to_box(x1, y1, x2, y2);
                });
            }

            set_selection_start.set(None);
            set_selection_current.set(None);
            ev.prevent_default();
        }
    };

    let on_mouseleave = move |_| {
        set_selection_start.set(None);
        set_selection_current.set(None);

        if let Some(setter) = set_cursor {
            setter.set(None);
        }
    };

    let on_dblclick = move |ev: web_sys::MouseEvent| {
        transform.set(original.get());
        ev.prevent_default();
    };

    view! {
        <>
            <rect
                width=move || inner_width.get()
                height=move || inner_height.get()
                fill="transparent"
                style="cursor: cell; pointer-events: all;"
                on:mousedown=on_mousedown
                on:mousemove=on_mousemove
                on:mouseup=on_mouseup
                on:mouseleave=on_mouseleave
                on:dblclick=on_dblclick
            />
            {move || {
                selection_rect
                    .get()
                    .map(|(x, y, w, h)| {
                        view! {
                            <rect
                                x=x
                                y=y
                                width=w
                                height=h
                                fill="rgba(66, 135, 245, 0.2)"
                                stroke="rgba(66, 135, 245, 0.8)"
                                stroke-width="1"
                                style="pointer-events: none;"
                            />
                        }
                    })
            }}
        </>
    }
}

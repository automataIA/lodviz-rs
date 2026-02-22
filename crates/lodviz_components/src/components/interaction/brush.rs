/// Brush selection overlay for charts
///
/// Renders a transparent SVG rect that captures mouse events.
/// On drag, draws a semi-transparent selection rectangle and emits
/// a `Selection::Interval` via callback on mouseup.
use leptos::prelude::*;
use lodviz_core::core::selection::Selection;

/// Brush selection overlay component
///
/// Place inside an SVG `<g>` positioned at the chart's inner area origin.
/// Captures mousedown/mousemove/mouseup to draw a selection rect.
#[component]
pub fn Brush(
    /// Callback fired when brush selection completes
    on_brush: Callback<Selection>,
    /// X domain min (current, after zoom)
    #[prop(into)]
    x_min: Signal<f64>,
    /// X domain max (current, after zoom)
    #[prop(into)]
    x_max: Signal<f64>,
    /// Y domain min (current, after zoom)
    #[prop(into)]
    y_min: Signal<f64>,
    /// Y domain max (current, after zoom)
    #[prop(into)]
    y_max: Signal<f64>,
    /// Inner width of the chart area in pixels
    #[prop(into)]
    inner_width: Signal<f64>,
    /// Inner height of the chart area in pixels
    #[prop(into)]
    inner_height: Signal<f64>,
    /// Brush color
    #[prop(default = "rgba(100,150,255,0.25)")]
    fill: &'static str,
    /// Brush border color
    #[prop(default = "rgba(100,150,255,0.6)")]
    stroke: &'static str,
) -> impl IntoView {
    let (is_brushing, set_is_brushing) = signal(false);
    let (start_px, set_start_px) = signal((0.0_f64, 0.0_f64));
    let (current_px, set_current_px) = signal((0.0_f64, 0.0_f64));

    // Compute brush rect in pixel space
    let brush_rect = Memo::new(move |_| {
        let (sx, sy) = start_px.get();
        let (cx, cy) = current_px.get();
        let rx = sx.min(cx);
        let ry = sy.min(cy);
        let rw = (cx - sx).abs();
        let rh = (cy - sy).abs();
        (rx, ry, rw, rh)
    });

    let on_mousedown = move |ev: web_sys::MouseEvent| {
        ev.prevent_default();
        let x = ev.offset_x() as f64;
        let y = ev.offset_y() as f64;
        set_start_px.set((x, y));
        set_current_px.set((x, y));
        set_is_brushing.set(true);
    };

    let on_mousemove = move |ev: web_sys::MouseEvent| {
        if !is_brushing.get() {
            return;
        }
        let x = (ev.offset_x() as f64).clamp(0.0, inner_width.get());
        let y = (ev.offset_y() as f64).clamp(0.0, inner_height.get());
        set_current_px.set((x, y));
    };

    let brush_fill = fill.to_string();
    let brush_stroke = stroke.to_string();

    let on_mouseup = move |_: web_sys::MouseEvent| {
        if !is_brushing.get() {
            return;
        }
        set_is_brushing.set(false);

        let w = inner_width.get();
        let h = inner_height.get();
        if w <= 0.0 || h <= 0.0 {
            return;
        }

        let (rx, ry, rw, rh) = brush_rect.get();
        // Minimum 4px drag to count as a brush
        if rw < 4.0 && rh < 4.0 {
            return;
        }

        // Convert pixel rect to domain coordinates
        let x_domain_min = x_min.get();
        let x_domain_max = x_max.get();
        let y_domain_min = y_min.get();
        let y_domain_max = y_max.get();
        let x_range = x_domain_max - x_domain_min;
        let y_range = y_domain_max - y_domain_min;

        let sel_x_min = x_domain_min + (rx / w) * x_range;
        let sel_x_max = x_domain_min + ((rx + rw) / w) * x_range;
        // SVG y is inverted: top of rect = higher y domain value
        let sel_y_max = y_domain_max - (ry / h) * y_range;
        let sel_y_min = y_domain_max - ((ry + rh) / h) * y_range;

        on_brush.run(Selection::interval_xy(
            sel_x_min, sel_x_max, sel_y_min, sel_y_max,
        ));
    };

    let on_mouseleave = move |_: web_sys::MouseEvent| {
        set_is_brushing.set(false);
    };

    view! {
        <g class="brush-overlay">
            <rect
                width=move || format!("{:.2}", inner_width.get())
                height=move || format!("{:.2}", inner_height.get())
                fill="transparent"
                style="cursor: crosshair;"
                on:mousedown=on_mousedown
                on:mousemove=on_mousemove
                on:mouseup=on_mouseup
                on:mouseleave=on_mouseleave
            />
            {move || {
                if is_brushing.get() {
                    let (rx, ry, rw, rh) = brush_rect.get();
                    view! {
                        <rect
                            x=format!("{rx:.2}")
                            y=format!("{ry:.2}")
                            width=format!("{rw:.2}")
                            height=format!("{rh:.2}")
                            fill=brush_fill.clone()
                            stroke=brush_stroke.clone()
                            stroke-width="1"
                            pointer-events="none"
                        />
                    }
                        .into_any()
                } else {
                    ().into_any()
                }
            }}
        </g>
    }
}

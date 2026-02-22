/// Reactive tooltip component for SVG charts (multi-series)
///
/// Shows crosshair + nearest point info per visible series on mouse hover.
/// Uses binary search via `find_nearest_point` for O(log n) lookup.
use leptos::prelude::*;
use lodviz_core::algorithms::nearest::find_nearest_point;
use lodviz_core::core::data::DataPoint;
use lodviz_core::core::scale::{LinearScale, Scale};

/// Tooltip selection mode
#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum TooltipMode {
    /// Select nearest point based on X-axis distance only (default)
    #[default]
    BisectX,
    /// Select nearest point based on 2D Euclidean distance
    Euclidean,
}

/// Tooltip overlay for chart interaction (multi-series)
#[component]
pub fn Tooltip(
    /// Series data: Vec of (name, points) for visible series
    series_data: Memo<Vec<(String, Vec<DataPoint>)>>,
    /// Colors for each visible series (parallel to series_data)
    series_colors: Memo<Vec<String>>,
    /// X scale for pixel-to-data conversion
    x_scale: Memo<LinearScale>,
    /// Y scale for pixel-to-data conversion
    y_scale: Memo<LinearScale>,
    /// Inner width of the chart area
    inner_width: Memo<f64>,
    /// Inner height of the chart area
    inner_height: Memo<f64>,
    /// Optional external cursor X position (normalized 0..1)
    #[prop(optional, into)]
    cursor_normalized_x: Option<Signal<Option<f64>>>,
    /// Optional external cursor Y position (normalized 0..1)
    #[prop(optional, into)]
    cursor_normalized_y: Option<Signal<Option<f64>>>,
    /// Tooltip selection mode
    #[prop(optional, default = TooltipMode::BisectX)]
    mode: TooltipMode,
) -> impl IntoView {
    // If external cursor is provided, use it. Otherwise use internal tracking (backward compat or standalone use).

    let (internal_mouse_x, set_internal_mouse_x) = signal(None::<f64>);
    let (internal_mouse_y, set_internal_mouse_y) = signal(None::<f64>);

    let effective_mouse_pos = Memo::new(move |_| {
        let mx = if let Some(cnx_sig) = cursor_normalized_x {
            cnx_sig.get().map(|nx| nx * inner_width.get())
        } else {
            internal_mouse_x.get()
        };

        let my = if let Some(cny_sig) = cursor_normalized_y {
            cny_sig.get().map(|ny| ny * inner_height.get())
        } else {
            internal_mouse_y.get()
        };

        match (mx, my) {
            (Some(x), Some(y)) => Some((x, y)),
            (Some(x), None) => Some((x, 0.0)), // Default Y for BisectX
            _ => None,
        }
    });

    // Find nearest point for each visible series (or single nearest for Euclidean)
    let nearest_points = Memo::new(move |_| {
        let (mx, my) = effective_mouse_pos.get()?;
        let series = series_data.get();
        let colors = series_colors.get();
        let xs = x_scale.get();
        let ys = y_scale.get();

        match mode {
            TooltipMode::BisectX => {
                let data_x = xs.inverse(mx);
                let results: Vec<(String, String, DataPoint)> = series
                    .iter()
                    .zip(colors.iter())
                    .filter_map(|((name, pts), color)| {
                        let (_, pt) = find_nearest_point(pts, data_x)?;
                        Some((name.clone(), color.clone(), *pt))
                    })
                    .collect();

                if results.is_empty() {
                    None
                } else {
                    Some(results)
                }
            }
            TooltipMode::Euclidean => {
                // Find single closest point across ALL series
                let mut min_dist_sq = f64::MAX;
                let mut best_match: Option<(String, String, DataPoint)> = None;

                // Maximum distance in pixels to trigger tooltip (e.g., 50px radius)
                let max_dist_sq = 50.0 * 50.0;

                for ((name, pts), color) in series.iter().zip(colors.iter()) {
                    for pt in pts {
                        let px = xs.map(pt.x);
                        let py = ys.map(pt.y);
                        let dx = px - mx;
                        let dy = py - my;
                        let dist_sq = dx * dx + dy * dy;

                        if dist_sq < min_dist_sq && dist_sq < max_dist_sq {
                            min_dist_sq = dist_sq;
                            best_match = Some((name.clone(), color.clone(), *pt));
                        }
                    }
                }

                best_match.map(|m| vec![m])
            }
        }
    });

    view! {
        // Invisible rect to capture mouse events over the entire chart area
        // Only render if external cursor is NOT provided (fallback mode)
        {move || {
            if cursor_normalized_x.is_none() {
                Some(
                    view! {
                        <rect
                            width=move || inner_width.get()
                            height=move || inner_height.get()
                            fill="transparent"
                            style="pointer-events: all;"
                            on:mousemove=move |ev| {
                                let x = ev.offset_x() as f64;
                                let y = ev.offset_y() as f64;
                                set_internal_mouse_x.set(Some(x));
                                set_internal_mouse_y.set(Some(y));
                            }
                            on:mouseleave=move |_| {
                                set_internal_mouse_x.set(None);
                                set_internal_mouse_y.set(None);
                            }
                        />
                    },
                )
            } else {
                None
            }
        }}
        // Crosshair + highlight + tooltip box
        {move || {
            let results = nearest_points.get()?;
            let xs = x_scale.get();
            let ys = y_scale.get();
            let h = inner_height.get();
            let w = inner_width.get();
            let first = &results[0];
            let cx = xs.map(first.2.x);
            let n_series = results.len();
            let box_h = 20.0 + n_series as f64 * 18.0;
            let box_w = 120.0_f64;
            let box_x = if cx > w * 0.7 { cx - box_w - 10.0 } else { cx + 10.0 };
            let first_cy = ys.map(first.2.y);
            let box_y = if first_cy > h * 0.7 { first_cy - box_h - 10.0 } else { first_cy - 10.0 };
            Some(

                // Use the first series point for crosshair x position

                // Tooltip box sizing

                view! {
                    <g class="tooltip-overlay" style="pointer-events: none;">
                        // Vertical crosshair
                        <line
                            x1=format!("{cx:.2}")
                            y1="0"
                            x2=format!("{cx:.2}")
                            y2=format!("{h:.2}")
                            stroke="#999"
                            stroke-width="1"
                            stroke-dasharray="4,4"
                            opacity="0.6"
                        />

                        // Highlight circles on each series point
                        {results
                            .iter()
                            .map(|(_, color, pt)| {
                                let px = xs.map(pt.x);
                                let py = ys.map(pt.y);
                                view! {
                                    <circle
                                        cx=format!("{px:.2}")
                                        cy=format!("{py:.2}")
                                        r="5"
                                        fill="white"
                                        stroke=color.clone()
                                        stroke-width="2"
                                    />
                                }
                            })
                            .collect_view()}

                        // Tooltip background
                        <rect
                            x=format!("{box_x:.2}")
                            y=format!("{box_y:.2}")
                            width=box_w
                            height=box_h
                            rx="4"
                            fill="rgba(0,0,0,0.8)"
                        />

                        // Header: x value
                        <text
                            x=format!("{:.2}", box_x + 8.0)
                            y=format!("{:.2}", box_y + 14.0)
                            font-size="11"
                            fill="white"
                            font-family="monospace"
                            font-weight="bold"
                        >
                            {format!("x: {:.2}", first.2.x)}
                        </text>

                        // One row per series
                        {results
                            .iter()
                            .enumerate()
                            .map(|(i, (name, color, pt))| {
                                let ty = box_y + 28.0 + i as f64 * 18.0;
                                view! {
                                    <g>
                                        // Color dot
                                        <circle
                                            cx=format!("{:.2}", box_x + 12.0)
                                            cy=format!("{:.2}", ty - 3.0)
                                            r="4"
                                            fill=color.clone()
                                        />
                                        // Series name + value
                                        <text
                                            x=format!("{:.2}", box_x + 20.0)
                                            y=format!("{ty:.2}")
                                            font-size="10"
                                            fill="white"
                                            font-family="monospace"
                                        >
                                            {format!("{}: {:.2}", name, pt.y)}
                                        </text>
                                    </g>
                                }
                            })
                            .collect_view()}
                    </g>
                },
            )
        }}
    }
}

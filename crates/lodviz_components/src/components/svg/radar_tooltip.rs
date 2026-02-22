/// Tooltip overlay for radar (spider) charts with euclidean distance hit-testing
use leptos::prelude::*;
use lodviz_core::core::theme::Margin;

/// Tooltip data for a single radar series
#[derive(Clone, Debug, PartialEq)]
pub struct RadarTooltipSeries {
    /// Series name shown in header
    pub name: String,
    /// Series color (hex string)
    pub color: String,
    /// Pixel-space coordinates of each vertex (one per axis)
    pub vertices: Vec<(f64, f64)>,
    /// Raw data values (one per axis)
    pub values: Vec<f64>,
}

/// Tooltip overlay for radar charts
///
/// Hit-tests by finding the series vertex nearest to the mouse cursor
/// within 25% of the radar radius.
#[component]
pub fn RadarTooltip(
    /// All radar series with pre-computed pixel vertices
    series: Memo<Vec<RadarTooltipSeries>>,
    /// Axis labels (static, one per spoke)
    axes: Vec<String>,
    /// Pixel radius of the outermost ring
    radius: Memo<f64>,
    /// Inner width of the chart area
    inner_width: Memo<f64>,
    /// Inner height of the chart area
    inner_height: Memo<f64>,
    /// Chart margins (to correct SVG offset coordinates)
    margin: Memo<Margin>,
) -> impl IntoView {
    let (mouse_pos, set_mouse_pos) = signal(None::<(f64, f64)>);

    // Find the series whose nearest vertex is within 25% of radius
    let hovered_series = Memo::new(move |_| {
        let (mx, my) = mouse_pos.get()?;
        let all = series.get();
        let max_dist_sq = (radius.get() * 0.25).powi(2);
        all.iter()
            .enumerate()
            .flat_map(|(i, s)| {
                s.vertices
                    .iter()
                    .map(move |&(vx, vy)| (i, (vx - mx).powi(2) + (vy - my).powi(2)))
            })
            .filter(|&(_, d)| d < max_dist_sq)
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
    });

    let n_axes = axes.len();

    view! {
        // Transparent overlay to capture mouse events
        <rect
            width=move || inner_width.get()
            height=move || inner_height.get()
            fill="transparent"
            style="pointer-events: all;"
            on:mousemove=move |ev| {
                let m = margin.get();
                let x = ev.offset_x() as f64 - m.left;
                let y = ev.offset_y() as f64 - m.top;
                set_mouse_pos.set(Some((x, y)));
            }
            on:mouseleave=move |_| {
                set_mouse_pos.set(None);
            }
        />

        // Tooltip rendering (pointer-events disabled)
        {move || {
            let si = hovered_series.get()?;
            let (mx, my) = mouse_pos.get()?;
            let all = series.get();
            let s = all.get(si)?;
            let w = inner_width.get();
            let h = inner_height.get();
            let pts_str: String = s
                .vertices
                .iter()
                .map(|(x, y)| format!("{x:.2},{y:.2}"))
                .collect::<Vec<_>>()
                .join(" ");
            let box_w = 160.0_f64;
            let padding = 8.0_f64;
            let header_h = 18.0_f64;
            let row_h = 16.0_f64;
            let box_h = padding * 2.0 + header_h + n_axes as f64 * row_h;
            let box_x = if mx + box_w + 10.0 > w { mx - box_w - 10.0 } else { mx + 10.0 };
            let box_y = if my + box_h + 10.0 > h { my - box_h - 10.0 } else { my + 10.0 };
            let name = s.name.clone();
            let color = s.color.clone();
            let values = s.values.clone();
            Some(

                // Build polygon points string for the hovered series highlight

                // Tooltip box sizing

                // Auto-flip near edges

                view! {
                    <g class="radar-tooltip-overlay" style="pointer-events: none;">
                        // Hovered polygon highlight (thicker stroke, semi-transparent)
                        <polygon
                            points=pts_str
                            fill=format!("{}22", color)
                            stroke=color.clone()
                            stroke-width="3"
                            stroke-linejoin="round"
                            opacity="0.8"
                        />

                        // Tooltip box background
                        <rect
                            x=format!("{box_x:.2}")
                            y=format!("{box_y:.2}")
                            width=box_w
                            height=box_h
                            rx="4"
                            fill="rgba(0,0,0,0.85)"
                        />

                        // Left color stripe
                        <rect
                            x=format!("{box_x:.2}")
                            y=format!("{box_y:.2}")
                            width="3"
                            height=box_h
                            rx="4"
                            fill=color.clone()
                        />

                        // Header: series name
                        <text
                            x=format!("{:.2}", box_x + padding)
                            y=format!("{:.2}", box_y + padding + 12.0)
                            font-size="11"
                            fill="white"
                            font-family="monospace"
                            font-weight="bold"
                        >
                            {name}
                        </text>

                        // Axis value rows
                        {axes
                            .iter()
                            .enumerate()
                            .map(|(i, axis_label)| {
                                let val = values.get(i).copied().unwrap_or(0.0);
                                let ty = box_y + padding + header_h + row_h * (i as f64 + 1.0);
                                view! {
                                    <text
                                        x=format!("{:.2}", box_x + padding)
                                        y=format!("{ty:.2}")
                                        font-size="10"
                                        fill="#ddd"
                                        font-family="monospace"
                                    >
                                        {format!("{}: {val:.2}", axis_label)}
                                    </text>
                                }
                            })
                            .collect_view()}
                    </g>
                },
            )
        }}
    }
}

/// Tooltip overlay for BoxPlot and ViolinChart with BandScale hit-testing
use leptos::prelude::*;
use lodviz_core::algorithms::statistics::BoxPlotStats;
use lodviz_core::core::scale::BandScale;
use lodviz_core::core::theme::Margin;

/// Tooltip data for a single box/violin group
#[derive(Clone, Debug, PartialEq)]
pub struct BoxGroupTooltipData {
    /// The label of the category/group
    pub label: String,
    /// Pixel x-center of the group band
    pub center: f64,
    /// Pixel width of the band
    pub band_width: f64,
    /// Box plot statistics (domain values)
    pub stats: BoxPlotStats,
    /// Number of raw data points in the group
    pub n: usize,
    /// Series color (hex string)
    pub color: String,
}

/// Tooltip overlay for BoxPlot and ViolinChart
#[component]
pub fn BoxViolinTooltip(
    /// Tooltip data for each group (in band order)
    groups: Memo<Vec<BoxGroupTooltipData>>,
    /// Band scale for hit-testing
    band_scale: Memo<BandScale>,
    /// Inner width of the chart area
    inner_width: Memo<f64>,
    /// Inner height of the chart area
    inner_height: Memo<f64>,
    /// Chart margins (to correct SVG offset coordinates)
    margin: Memo<Margin>,
) -> impl IntoView {
    let (mouse_pos, set_mouse_pos) = signal(None::<(f64, f64)>);

    // BandScale step-based hit-test (same pattern as BarTooltip)
    let hovered_idx = Memo::new(move |_| {
        let (mx, _) = mouse_pos.get()?;
        let bs = band_scale.get();
        let (r0, r1) = bs.range();
        let step = bs.step();
        if step <= 0.0 {
            return None;
        }
        let range_min = r0.min(r1);
        let range_max = r0.max(r1);
        if mx < range_min || mx > range_max {
            return None;
        }
        let idx = ((mx - range_min) / step).floor() as usize;
        let n = bs.len();
        if n == 0 {
            return None;
        }
        Some(idx.min(n - 1))
    });

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
            let idx = hovered_idx.get()?;
            let (mx, my) = mouse_pos.get()?;
            let all_groups = groups.get();
            let g = all_groups.get(idx)?;
            let w = inner_width.get();
            let h = inner_height.get();
            let s = &g.stats;
            let n_outliers = s.outliers.len();
            let hl_x = g.center - g.band_width / 2.0;
            let box_w = 200.0_f64;
            let padding = 8.0_f64;
            let header_h = 18.0_f64;
            let row_h = 14.0_f64;
            let n_rows = 5 + if n_outliers > 0 { 1 } else { 0 };
            let box_h = padding * 2.0 + header_h + n_rows as f64 * row_h;
            let box_x = if mx + box_w + 10.0 > w { mx - box_w - 10.0 } else { mx + 10.0 };
            let box_y = if my + box_h + 10.0 > h { my - box_h - 10.0 } else { my + 10.0 };
            let label = g.label.clone();
            let color = g.color.clone();
            let n_pts = g.n;
            let min_v = s.lower_whisker;
            let max_v = s.upper_whisker;
            let q1 = s.q1;
            let q3 = s.q3;
            let median = s.median;
            let mean = s.mean;
            let iqr = s.iqr;
            Some(

                // Highlight band covering full height of chart

                // Tooltip box sizing (dynamic height for optional outliers row)

                // Auto-flip near edges

                // Pre-format strings for the view

                view! {
                    <g class="box-violin-tooltip-overlay" style="pointer-events: none;">
                        // Band highlight
                        <rect
                            x=format!("{hl_x:.2}")
                            y="0"
                            width=format!("{:.2}", g.band_width)
                            height=format!("{h:.2}")
                            fill="white"
                            opacity="0.15"
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

                        // Header: label + N
                        <text
                            x=format!("{:.2}", box_x + padding)
                            y=format!("{:.2}", box_y + padding + 12.0)
                            font-size="11"
                            fill="white"
                            font-family="monospace"
                            font-weight="bold"
                        >
                            {format!("{label}  (N={n_pts})")}
                        </text>

                        // Min / Max row
                        <text
                            x=format!("{:.2}", box_x + padding)
                            y=format!("{:.2}", box_y + padding + header_h + row_h)
                            font-size="10"
                            fill="#ddd"
                            font-family="monospace"
                        >
                            {format!("Min: {min_v:.2}   Max: {max_v:.2}")}
                        </text>

                        // Q1 / Q3 row
                        <text
                            x=format!("{:.2}", box_x + padding)
                            y=format!("{:.2}", box_y + padding + header_h + row_h * 2.0)
                            font-size="10"
                            fill="#ddd"
                            font-family="monospace"
                        >
                            {format!("Q1:  {q1:.2}   Q3:  {q3:.2}")}
                        </text>

                        // Median / Mean row
                        <text
                            x=format!("{:.2}", box_x + padding)
                            y=format!("{:.2}", box_y + padding + header_h + row_h * 3.0)
                            font-size="10"
                            fill="#ddd"
                            font-family="monospace"
                        >
                            {format!("Med: {median:.2}   Mean: {mean:.2}")}
                        </text>

                        // IQR row
                        <text
                            x=format!("{:.2}", box_x + padding)
                            y=format!("{:.2}", box_y + padding + header_h + row_h * 4.0)
                            font-size="10"
                            fill="#ddd"
                            font-family="monospace"
                        >
                            {format!("IQR: {iqr:.2}")}
                        </text>

                        // Optional outliers row
                        {if n_outliers > 0 {
                            let ty = box_y + padding + header_h + row_h * 5.0;
                            Some(
                                view! {
                                    <text
                                        x=format!("{:.2}", box_x + padding)
                                        y=format!("{ty:.2}")
                                        font-size="10"
                                        fill="#f9a825"
                                        font-family="monospace"
                                    >
                                        {format!("Outliers: {n_outliers}")}
                                    </text>
                                },
                            )
                        } else {
                            None
                        }}
                    </g>
                },
            )
        }}
    }
}

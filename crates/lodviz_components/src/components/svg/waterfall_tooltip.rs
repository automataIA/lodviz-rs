/// Tooltip overlay for waterfall charts with BandScale hit-testing
use leptos::prelude::*;
use lodviz_core::core::data::WaterfallKind;
use lodviz_core::core::scale::BandScale;
use lodviz_core::core::theme::Margin;

/// A single tooltip entry for a waterfall bar
#[derive(Clone, Debug, PartialEq)]
pub struct WaterfallTooltipEntry {
    /// Category / step label
    pub label: String,
    /// Signed delta (original bar value)
    pub value: f64,
    /// Running cumulative total at this bar
    pub running_total: f64,
    /// The bar kind (Start, Delta, Total)
    pub kind: WaterfallKind,
    /// The CSS color of the bar
    pub color: String,
}

/// Tooltip overlay for waterfall charts
#[component]
pub fn WaterfallTooltip(
    /// Tooltip entries (one per waterfall bar, in order)
    entries: Memo<Vec<WaterfallTooltipEntry>>,
    /// Band scale for the X axis
    band_scale: Memo<BandScale>,
    /// Inner width of the chart area
    inner_width: Memo<f64>,
    /// Inner height of the chart area
    inner_height: Memo<f64>,
    /// Chart margins (to correct SVG offset coordinates)
    margin: Memo<Margin>,
) -> impl IntoView {
    let (mouse_pos, set_mouse_pos) = signal(None::<(f64, f64)>);

    // Determine which bar index is hovered
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

        // Tooltip rendering (pointer-events disabled so it doesn't block hover)
        {move || {
            let idx = hovered_idx.get()?;
            let (mx, my) = mouse_pos.get()?;
            let all_entries = entries.get();
            let entry = all_entries.into_iter().nth(idx)?;
            let bs = band_scale.get();
            let w = inner_width.get();
            let h = inner_height.get();
            let hl_x = bs.map_index(idx);
            let hl_w = bs.band_width();
            let kind_label = match entry.kind {
                WaterfallKind::Start => "Start",
                WaterfallKind::Delta => "Δ",
                WaterfallKind::Total => "Total",
            };
            let value_prefix = if entry.value >= 0.0 { "+" } else { "" };
            let box_w = 175.0_f64;
            let box_h = 70.0_f64;
            let padding = 8.0_f64;
            let row_h = 16.0_f64;
            let box_x = if mx + box_w + 10.0 > w { mx - box_w - 10.0 } else { mx + 10.0 };
            let box_y = if my + box_h + 10.0 > h { my - box_h - 10.0 } else { my + 10.0 };
            Some(

                // Highlight band covering full height of chart

                // Kind badge text

                // Value prefix sign

                // Tooltip box sizing

                // Auto-flip near edges

                view! {
                    <g class="waterfall-tooltip-overlay" style="pointer-events: none;">
                        // Vertical band highlight
                        <rect
                            x=format!("{hl_x:.2}")
                            y="0"
                            width=format!("{hl_w:.2}")
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

                        // Color indicator
                        <rect
                            x=format!("{:.2}", box_x)
                            y=format!("{box_y:.2}")
                            width="3"
                            height=box_h
                            rx="4"
                            fill=entry.color.clone()
                        />

                        // Header: label + kind badge
                        <text
                            x=format!("{:.2}", box_x + padding)
                            y=format!("{:.2}", box_y + padding + 12.0)
                            font-size="11"
                            fill="white"
                            font-family="monospace"
                            font-weight="bold"
                        >
                            {format!("{} [{}]", entry.label, kind_label)}
                        </text>

                        // Value row
                        <text
                            x=format!("{:.2}", box_x + padding)
                            y=format!("{:.2}", box_y + padding + 12.0 + row_h)
                            font-size="10"
                            fill="#ddd"
                            font-family="monospace"
                        >
                            {format!("Value:  {value_prefix}{:.2}", entry.value)}
                        </text>

                        // Running total row
                        <text
                            x=format!("{:.2}", box_x + padding)
                            y=format!("{:.2}", box_y + padding + 12.0 + row_h * 2.0)
                            font-size="10"
                            fill="#ddd"
                            font-family="monospace"
                        >
                            {format!("Total:  {:.2}", entry.running_total)}
                        </text>
                    </g>
                },
            )
        }}
    }
}

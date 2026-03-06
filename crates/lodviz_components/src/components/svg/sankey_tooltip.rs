/// Tooltip component for Sankey nodes and links
use leptos::prelude::*;

/// Floating tooltip for Sankey diagram hover events.
#[component]
pub fn SankeyTooltip(
    /// Hovered label (node label or "A → B" for links). None = hidden.
    label: Signal<Option<String>>,
    /// Hovered value. None = hidden.
    value: Signal<Option<f64>>,
    /// Mouse X position in chart-inner coordinates
    x: Signal<f64>,
    /// Mouse Y position in chart-inner coordinates
    y: Signal<f64>,
    /// Inner width of the chart area (for edge-flipping)
    inner_width: Signal<f64>,
    /// Inner height of the chart area (for edge-flipping)
    inner_height: Signal<f64>,
    /// Text color
    text_color: String,
) -> impl IntoView {
    view! {
        {move || {
            let lbl = label.get()?;
            let val = value.get()?;
            let mx = x.get();
            let my = y.get();
            let w = inner_width.get();
            let h = inner_height.get();
            let tc = text_color.clone();
            let val_str = format!("{val:.1}");
            let max_len = lbl.len().max(val_str.len() + 8);
            let box_w = 12.0 + max_len as f64 * 6.5;
            let box_h = 38.0;
            let bx = if mx + box_w + 12.0 > w { mx - box_w - 12.0 } else { mx + 12.0 };
            let by = if my + box_h + 8.0 > h { my - box_h - 8.0 } else { my + 8.0 };
            Some(

                view! {
                    <g class="sankey-tooltip" style="pointer-events: none;">
                        <rect
                            x=format!("{bx:.2}")
                            y=format!("{by:.2}")
                            width=format!("{box_w:.2}")
                            height=box_h
                            rx="4"
                            fill="rgba(0,0,0,0.85)"
                        />
                        <text
                            x=format!("{:.2}", bx + 8.0)
                            y=format!("{:.2}", by + 15.0)
                            font-size="11"
                            fill=tc.clone()
                            font-family="monospace"
                            font-weight="bold"
                        >
                            {lbl.clone()}
                        </text>
                        <text
                            x=format!("{:.2}", bx + 8.0)
                            y=format!("{:.2}", by + 30.0)
                            font-size="11"
                            fill=tc.clone()
                            font-family="monospace"
                        >
                            {format!("Value: {}", val_str)}
                        </text>
                    </g>
                },
            )
        }}
    }
}

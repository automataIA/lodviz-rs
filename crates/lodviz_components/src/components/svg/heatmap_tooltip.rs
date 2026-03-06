/// Tooltip component for heatmap cells
use leptos::prelude::*;

/// Floating tooltip that displays the row/column label and value for a hovered heatmap cell.
#[component]
pub fn HeatmapTooltip(
    /// Hovered row label (None = hidden)
    row_label: Signal<Option<String>>,
    /// Hovered column label (None = hidden)
    col_label: Signal<Option<String>>,
    /// Hovered cell value (None = hidden)
    value: Signal<Option<f64>>,
    /// Mouse X position in chart-inner coordinates
    x: Signal<f64>,
    /// Mouse Y position in chart-inner coordinates
    y: Signal<f64>,
    /// Inner width of the chart area (used for edge-flipping)
    inner_width: Signal<f64>,
    /// Inner height of the chart area (used for edge-flipping)
    inner_height: Signal<f64>,
    /// Text color
    text_color: String,
) -> impl IntoView {
    view! {
        {move || {
            let row = row_label.get()?;
            let col = col_label.get()?;
            let val = value.get()?;
            let mx = x.get();
            let my = y.get();
            let w = inner_width.get();
            let h = inner_height.get();
            let tc = text_color.clone();
            let label = format!("({row}, {col}): {val:.3}");
            let box_w = 8.0 + (label.len() as f64) * 6.5;
            let box_h = 28.0;
            let bx = if mx + box_w + 12.0 > w { mx - box_w - 12.0 } else { mx + 12.0 };
            let by = if my + box_h + 8.0 > h { my - box_h - 8.0 } else { my + 8.0 };
            Some(

                view! {
                    <g class="heatmap-tooltip" style="pointer-events: none;">
                        <rect
                            x=format!("{bx:.2}")
                            y=format!("{by:.2}")
                            width=format!("{box_w:.2}")
                            height=box_h
                            rx="4"
                            fill="rgba(0,0,0,0.82)"
                        />
                        <text
                            x=format!("{:.2}", bx + 8.0)
                            y=format!("{:.2}", by + 18.0)
                            font-size="11"
                            fill=tc.clone()
                            font-family="monospace"
                        >
                            {label}
                        </text>
                    </g>
                },
            )
        }}
    }
}

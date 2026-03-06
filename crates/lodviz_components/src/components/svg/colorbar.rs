/// ColorBar component: a vertical gradient bar with tick labels
use leptos::prelude::*;
use lodviz_core::core::color_map::ColorMap;

/// A vertical color legend (gradient bar + tick labels) for continuous color scales.
#[component]
pub fn ColorBar(
    /// The color map to display
    color_map: ColorMap,
    /// Minimum data value (shown at bottom)
    min_value: f64,
    /// Maximum data value (shown at top)
    max_value: f64,
    /// Width of the gradient rectangle in SVG units
    #[prop(default = 16.0)]
    bar_width: f64,
    /// Height of the gradient rectangle in SVG units
    height: f64,
    /// Number of tick labels to render
    #[prop(default = 5)]
    tick_count: usize,
    /// Text color for tick labels
    text_color: String,
    /// Font size for tick labels
    font_size: f64,
) -> impl IntoView {
    // Build N evenly-spaced gradient stops
    let n_stops = 20_usize;
    let stops: Vec<(f64, String)> = (0..=n_stops)
        .map(|i| {
            let t = i as f64 / n_stops as f64;
            // SVG linearGradient with gradientTransform="rotate(90)" needs t=0 at top
            // so we invert: high value → top (t=0 in CSS gradient = top)
            let color = color_map.map(1.0 - t);
            (t * 100.0, color)
        })
        .collect();

    let gradient_id = format!("colorbar-grad-{}", uuid::Uuid::new_v4().simple());
    let gradient_id2 = gradient_id.clone();

    // Tick positions: evenly spaced from top (max) to bottom (min)
    let ticks: Vec<(f64, String)> = (0..tick_count)
        .map(|i| {
            let t = if tick_count <= 1 {
                0.5
            } else {
                i as f64 / (tick_count - 1) as f64
            };
            let value = max_value - t * (max_value - min_value);
            let y = t * height;
            let label = if (value.abs() >= 1000.0) || (value != 0.0 && value.abs() < 0.01) {
                format!("{value:.2e}")
            } else {
                format!("{value:.2}")
            };
            (y, label)
        })
        .collect();

    let label_x = bar_width + 4.0;

    view! {
        <g class="color-bar">
            <defs>
                <linearGradient id=gradient_id gradientTransform="rotate(90)">
                    {stops
                        .iter()
                        .map(|(offset, color)| {
                            view! {
                                <stop offset=format!("{offset:.1}%") stop-color=color.clone() />
                            }
                        })
                        .collect_view()}
                </linearGradient>
            </defs>

            // Gradient rectangle
            <rect
                x="0"
                y="0"
                width=bar_width
                height=height
                fill=format!("url(#{})", gradient_id2)
            />

            // Border
            <rect
                x="0"
                y="0"
                width=bar_width
                height=height
                fill="none"
                stroke=text_color.clone()
                stroke-opacity="0.3"
                stroke-width="0.5"
            />

            // Tick labels
            {ticks
                .iter()
                .map(|(y, label)| {
                    let tc = text_color.clone();
                    view! {
                        <g>
                            // Small tick line
                            <line
                                x1=bar_width
                                y1=format!("{y:.2}")
                                x2=format!("{:.2}", bar_width + 3.0)
                                y2=format!("{y:.2}")
                                stroke=tc.clone()
                                stroke-width="0.5"
                            />
                            <text
                                x=format!("{label_x:.2}")
                                y=format!("{:.2}", y + font_size * 0.35)
                                font-size=font_size
                                fill=tc.clone()
                                font-family="monospace"
                            >
                                {label.clone()}
                            </text>
                        </g>
                    }
                })
                .collect_view()}
        </g>
    }
}

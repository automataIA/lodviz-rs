/// Interactive legend component for SVG charts
///
/// Displays series names with color swatches as an SVG overlay inside the chart.
/// Supports click-to-toggle visibility and automatic multi-column layout.
use leptos::prelude::*;

/// A single legend entry
#[derive(Clone, Debug, PartialEq)]
pub struct LegendItem {
    /// The name of the series displayed
    pub name: String,
    /// The color swatch associated with this series
    pub color: String,
    /// Whether the series is currently visible
    pub visible: bool,
}

/// Position of the legend within or adjacent to the chart area
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LegendPosition {
    #[default]
    /// Overlaid in the top-right corner of the inner chart area
    TopRight,
    /// Overlaid in the top-left corner of the inner chart area
    TopLeft,
    /// Overlaid in the bottom-right corner of the inner chart area
    BottomRight,
    /// Overlaid in the bottom-left corner of the inner chart area
    BottomLeft,
    /// Adjacent to the right edge of the inner chart area (in the right margin)
    ExternalRight,
}

/// Estimate the pixel width needed for a single-column legend panel.
///
/// Charts use this to pre-allocate right-margin space when `legend_outside` is enabled.
pub fn estimate_legend_width(items: &[LegendItem]) -> f64 {
    const PADDING: f64 = 6.0;
    const SWATCH: f64 = 10.0;
    const CHAR_W: f64 = 6.5;
    let max_len = items.iter().map(|i| i.name.len()).max().unwrap_or(4) as f64;
    SWATCH + 6.0 + max_len * CHAR_W + PADDING * 2.0
}

/// Legend component for chart series (SVG-internal)
///
/// Renders a list of series with color swatches. Clicking an item
/// fires `on_toggle(index)` to show/hide the corresponding series.
///
/// Columns are computed automatically: a second column is added when
/// the single-column height would exceed 70 % of `inner_height`.
#[component]
pub fn Legend(
    /// Legend items (name, color, visibility)
    items: Signal<Vec<LegendItem>>,
    /// Position within or adjacent to the inner chart area
    #[prop(default = LegendPosition::TopRight)]
    position: LegendPosition,
    /// Force a fixed column count (default: auto based on inner_height)
    #[prop(optional)]
    columns: Option<usize>,
    /// Inner width of the chart (used for right-aligned positioning)
    inner_width: Memo<f64>,
    /// Inner height of the chart (used for bottom-aligned and auto-column logic)
    inner_height: Memo<f64>,
    /// Callback when an item is toggled
    #[prop(optional, into)]
    on_toggle: Option<Callback<usize>>,
    /// Text color
    #[prop(default = "#333".to_string(), into)]
    text_color: String,
) -> impl IntoView {
    let item_height = 18.0_f64;
    let padding = 6.0_f64;
    let swatch_size = 10.0_f64;

    // Auto-compute the number of columns based on available height.
    // A second column is added when items would overflow 70 % of inner_height.
    let get_cols = move || {
        if let Some(fixed) = columns {
            return fixed.max(1);
        }
        let n = items.get().len();
        let ih = inner_height.get();
        let max_single = ((ih * 0.7 - padding * 2.0) / item_height).floor().max(1.0) as usize;
        if n <= max_single {
            1
        } else {
            2
        }
    };

    view! {
        <g
            class="legend"
            role="list"
            aria-label="Chart legend"
            style="pointer-events: none;"
            transform=move || {
                let items_vec = items.get();
                let n = items_vec.len();
                let cols = get_cols();
                let rows_per_col = n.div_ceil(cols);
                let box_h = rows_per_col as f64 * item_height + padding * 2.0;
                let max_name_len = items_vec.iter().map(|i| i.name.len()).max().unwrap_or(0) as f64;
                let col_w = swatch_size + 6.0 + max_name_len * 6.5 + padding;
                let box_w = col_w * cols as f64 + padding;
                let iw = inner_width.get();
                let ih = inner_height.get();
                let (x, y) = match position {
                    LegendPosition::TopRight => (iw - box_w - 8.0, 8.0),
                    LegendPosition::TopLeft => (8.0, 8.0),
                    LegendPosition::BottomRight => (iw - box_w - 8.0, ih - box_h - 8.0),
                    LegendPosition::BottomLeft => (8.0, ih - box_h - 8.0),
                    LegendPosition::ExternalRight => (iw + 8.0, 8.0),
                };
                format!("translate({x:.1}, {y:.1})")
            }
        >

            // Background (semi-transparent to show lines underneath)
            <rect
                width=move || {
                    let max_name_len = items.get().iter().map(|i| i.name.len()).max().unwrap_or(0)
                        as f64;
                    let cols = get_cols();
                    let col_w = swatch_size + 6.0 + max_name_len * 6.5 + padding;
                    col_w * cols as f64 + padding
                }

                height=move || {
                    let n = items.get().len();
                    let cols = get_cols();
                    let rows_per_col = n.div_ceil(cols);
                    rows_per_col as f64 * item_height + padding * 2.0
                }
                fill="rgba(255,255,255,0.85)"
                stroke="#ddd"
                stroke-width="1"
                rx=4
            />

            // Items arranged in columns
            {move || {
                let items_vec = items.get();
                let tc = text_color.clone();
                let n = items_vec.len();
                let cols = get_cols();
                let rows_per_col = n.div_ceil(cols);
                let max_name_len = items_vec.iter().map(|i| i.name.len()).max().unwrap_or(0) as f64;
                let col_w = swatch_size + 6.0 + max_name_len * 6.5 + padding;
                items_vec
                    .iter()
                    .enumerate()
                    .map(|(i, item)| {
                        let col = i / rows_per_col;
                        let row = i % rows_per_col;
                        let x_offset = col as f64 * col_w;
                        let y_pos = padding + row as f64 * item_height;
                        let opacity = if item.visible { "1" } else { "0.3" };
                        let color = item.color.clone();
                        let name = item.name.clone();
                        let fill_color = tc.clone();
                        view! {
                            <g
                                role="listitem"
                                style="cursor: pointer; pointer-events: all;"
                                opacity=opacity
                                on:click=move |_| {
                                    if let Some(cb) = on_toggle {
                                        cb.run(i);
                                    }
                                }
                            >

                                // Color swatch
                                <rect
                                    x=padding + x_offset
                                    y=y_pos + 2.0
                                    width=swatch_size
                                    height=swatch_size
                                    fill=color
                                    rx=2
                                />
                                // Series name
                                <text
                                    x=padding + x_offset + swatch_size + 4.0
                                    y=y_pos + swatch_size
                                    font-size="11"
                                    fill=fill_color
                                >
                                    {name}
                                </text>
                            </g>
                        }
                    })
                    .collect_view()
            }}
        </g>
    }
}

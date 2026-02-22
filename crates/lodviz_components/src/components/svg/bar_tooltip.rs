/// Tooltip component for bar charts with categorical hit-testing
///
/// Uses `BandScale` to determine which category is hovered,
/// then shows all visible series values for that category.
/// Supports grouped and stacked modes (with percentages and total).
use leptos::prelude::*;
use lodviz_core::core::scale::{BandScale, LinearScale};
use lodviz_core::core::theme::Margin;

use crate::components::charts::bar_chart::{BarMode, BarOrientation};

/// Series info passed to the bar tooltip
#[derive(Clone, Debug, PartialEq)]
pub struct BarTooltipSeries {
    /// The name of the series (used as row label in the tooltip)
    pub name: String,
    /// The values for each category index
    pub values: Vec<f64>,
    /// The color of the series
    pub color: String,
    /// Whether the series is currently visible
    pub visible: bool,
}

/// Tooltip overlay for bar charts (categorical axis)
#[component]
pub fn BarTooltip(
    /// Category labels
    categories: Memo<Vec<String>>,
    /// Series information (name, values, color, visibility)
    series_info: Memo<Vec<BarTooltipSeries>>,
    /// Band scale for the categorical axis
    band_scale: Memo<BandScale>,
    /// Linear scale for the value axis
    #[allow(unused)]
    value_scale: Memo<LinearScale>,
    /// Inner width of the chart area
    inner_width: Memo<f64>,
    /// Inner height of the chart area
    inner_height: Memo<f64>,
    /// Bar orientation
    orientation: BarOrientation,
    /// Bar mode (grouped or stacked)
    mode: BarMode,
    /// Chart margins (to correct SVG offset coordinates)
    margin: Memo<Margin>,
) -> impl IntoView {
    let (mouse_pos, set_mouse_pos) = signal(None::<(f64, f64)>);

    // Determine which category index is hovered
    let hovered_category = Memo::new(move |_| {
        let (mx, my) = mouse_pos.get()?;
        let bs = band_scale.get();
        let (r0, r1) = bs.range();
        let step = bs.step();
        if step <= 0.0 {
            return None;
        }

        let coord = match orientation {
            BarOrientation::Vertical => mx,
            BarOrientation::Horizontal => my,
        };

        let range_min = r0.min(r1);
        let range_max = r0.max(r1);
        if coord < range_min || coord > range_max {
            return None;
        }

        let idx = ((coord - range_min) / step).floor() as usize;
        let n = bs.len();
        if n == 0 {
            return None;
        }
        Some(idx.min(n - 1))
    });

    // Build tooltip data for the hovered category
    let tooltip_data = Memo::new(move |_| {
        let ci = hovered_category.get()?;
        let cats = categories.get();
        let series = series_info.get();
        let cat_name = cats.get(ci)?.clone();

        let visible_entries: Vec<(String, f64, String)> = series
            .iter()
            .filter(|s| s.visible)
            .map(|s| {
                let val = s.values.get(ci).copied().unwrap_or(0.0);
                (s.name.clone(), val, s.color.clone())
            })
            .collect();

        if visible_entries.is_empty() {
            return None;
        }

        let total: f64 = visible_entries.iter().map(|(_, v, _)| v).sum();

        Some((cat_name, visible_entries, total))
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

        // Tooltip rendering
        {move || {
            let (cat_name, entries, total) = tooltip_data.get()?;
            let ci = hovered_category.get()?;
            let (mx, my) = mouse_pos.get()?;
            let bs = band_scale.get();
            let w = inner_width.get();
            let h = inner_height.get();
            let is_stacked = mode == BarMode::Stacked;
            let (hl_x, hl_y, hl_w, hl_h) = match orientation {
                BarOrientation::Vertical => {
                    let x = bs.map_index(ci);
                    let bw = bs.band_width();
                    (x, 0.0, bw, h)
                }
                BarOrientation::Horizontal => {
                    let y = bs.map_index(ci);
                    let bw = bs.band_width();
                    (0.0, y, w, bw)
                }
            };
            let n_rows = entries.len();
            let box_w: f64 = if is_stacked { 170.0 } else { 140.0 };
            let header_h = 18.0;
            let row_h = 16.0;
            let footer_h = if is_stacked { 18.0 } else { 0.0 };
            let padding = 8.0;
            let box_h = padding * 2.0 + header_h + n_rows as f64 * row_h + footer_h;
            let box_x = if mx + box_w + 10.0 > w { mx - box_w - 10.0 } else { mx + 10.0 };
            let box_y = if my + box_h + 10.0 > h { my - box_h - 10.0 } else { my + 10.0 };
            Some(

                // Highlight band

                // Tooltip box sizing

                // Position follows mouse with edge flip

                view! {
                    <g class="bar-tooltip-overlay" style="pointer-events: none;">
                        // Category band highlight
                        <rect
                            x=format!("{hl_x:.2}")
                            y=format!("{hl_y:.2}")
                            width=format!("{hl_w:.2}")
                            height=format!("{hl_h:.2}")
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

                        // Header: category name
                        <text
                            x=format!("{:.2}", box_x + padding)
                            y=format!("{:.2}", box_y + padding + 12.0)
                            font-size="11"
                            fill="white"
                            font-family="monospace"
                            font-weight="bold"
                        >
                            {cat_name}
                        </text>

                        // Series rows
                        {entries
                            .iter()
                            .enumerate()
                            .map(|(i, (name, val, color))| {
                                let ty = box_y + padding + header_h + 12.0 + i as f64 * row_h;
                                let label = if is_stacked && total > 0.0 {
                                    let pct = val / total * 100.0;
                                    format!("{name}: {val:.1} ({pct:.0}%)")
                                } else {
                                    format!("{name}: {val:.1}")
                                };
                                view! {
                                    <g>
                                        <circle
                                            cx=format!("{:.2}", box_x + padding + 4.0)
                                            cy=format!("{:.2}", ty - 3.0)
                                            r="4"
                                            fill=color.clone()
                                        />
                                        <text
                                            x=format!("{:.2}", box_x + padding + 12.0)
                                            y=format!("{ty:.2}")
                                            font-size="10"
                                            fill="white"
                                            font-family="monospace"
                                        >
                                            {label}
                                        </text>
                                    </g>
                                }
                            })
                            .collect_view()}

                        // Footer for stacked: total
                        {if is_stacked {
                            let footer_y = box_y + padding + header_h + n_rows as f64 * row_h
                                + 14.0;
                            Some(
                                view! {
                                    <text
                                        x=format!("{:.2}", box_x + padding)
                                        y=format!("{footer_y:.2}")
                                        font-size="10"
                                        fill="#aaa"
                                        font-family="monospace"
                                    >
                                        {format!("Total: {total:.1}")}
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

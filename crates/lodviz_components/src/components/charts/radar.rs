/// Radar (spider) chart component
use crate::components::svg::legend::{estimate_legend_width, Legend, LegendItem, LegendPosition};
use crate::components::svg::radar_tooltip::{RadarTooltip, RadarTooltipSeries};
use crate::hooks::use_container_size;
use leptos::prelude::*;
use lodviz_core::core::theme::ChartConfig;

/// A single data series for a radar chart
#[derive(Clone, Debug)]
pub struct RadarSeries {
    /// The name of the series (used for legend and tooltip)
    pub name: String,
    /// One value per axis (same length as `axes` in the component)
    pub values: Vec<f64>,
}

/// Convert polar (angle, radius) to Cartesian coordinates.
///
/// `angle_deg = 0` points up (−π/2 radians in standard math).
fn polar_to_cart(cx: f64, cy: f64, radius: f64, angle_rad: f64) -> (f64, f64) {
    (cx + radius * angle_rad.cos(), cy + radius * angle_rad.sin())
}

/// Build the SVG polygon points string from a list of (x, y) pairs.
fn polygon_points(pts: &[(f64, f64)]) -> String {
    pts.iter()
        .map(|(x, y)| format!("{x:.2},{y:.2}"))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Radar chart: concentric polygon grid + per-axis spokes + one polygon per series
///
/// Uses custom SVG layout (no Axis / Grid components — they don't apply here).
#[component]
pub fn RadarChart(
    /// Labels for each axis / spoke
    axes: Vec<String>,
    /// Data series (each must have `values.len() == axes.len()`)
    data: Signal<Vec<RadarSeries>>,
    /// Maximum domain value (auto-detected from data if not set)
    #[prop(optional)]
    max_value: Option<f64>,
    /// Number of concentric grid levels
    #[prop(default = 5)]
    grid_levels: usize,
    /// Chart configuration
    #[prop(default = Signal::derive(|| ChartConfig::default()), into)]
    config: Signal<ChartConfig>,
    /// Fixed width
    #[prop(optional)]
    width: Option<u32>,
    /// Fixed height
    #[prop(optional)]
    height: Option<u32>,
) -> impl IntoView {
    let theme = Memo::new(move |_| config.get().theme.unwrap_or_default());
    let (container_width, container_height, container_ref) = use_container_size();

    let chart_width = Memo::new(move |_| {
        let measured = container_width.get();
        if measured > 0.0 {
            return measured as u32;
        }
        config.get().width.or(width).unwrap_or(600)
    });

    let chart_height = Memo::new(move |_| {
        let measured = container_height.get();
        if measured > 0.0 {
            return measured as u32;
        }
        config.get().height.or(height).unwrap_or(400)
    });

    // Legend items — defined early so margin can adapt when legend_outside is enabled
    let legend_items = Signal::derive(move || {
        let series = data.get();
        let th = theme.get();
        series
            .iter()
            .enumerate()
            .map(|(i, s)| LegendItem {
                name: s.name.clone(),
                color: th.palette[i % th.palette.len()].clone(),
                visible: true,
            })
            .collect::<Vec<_>>()
    });

    let legend_outside = Memo::new(move |_| config.get().legend_outside.unwrap_or(false));

    let margin = Memo::new(move |_| {
        let mut m = config.get().margin.unwrap_or_default();
        if legend_outside.get() {
            m.right += estimate_legend_width(&legend_items.get()) + 16.0;
        }
        m
    });
    let inner_width =
        Memo::new(move |_| chart_width.get() as f64 - margin.get().left - margin.get().right);
    let inner_height =
        Memo::new(move |_| chart_height.get() as f64 - margin.get().top - margin.get().bottom);

    let final_title = Memo::new(move |_| config.get().title.clone());

    let n_axes = axes.len();
    let axes_clone = axes.clone();
    let axes_for_tooltip = axes.clone();

    // Maximum value (for scaling)
    let max_val = Memo::new(move |_| {
        max_value.unwrap_or_else(|| {
            data.get()
                .iter()
                .flat_map(|s| s.values.iter().copied())
                .fold(0.0_f64, f64::max)
                .max(1.0)
        })
    });

    // Spoke angle for axis i (start at top = -π/2)
    let spoke_angle = move |i: usize| {
        -std::f64::consts::FRAC_PI_2 + 2.0 * std::f64::consts::PI * i as f64 / n_axes as f64
    };

    // Radar center and radius as reactive memos (used by tooltip)
    let radar_center = Memo::new(move |_| {
        let iw = inner_width.get();
        let ih = inner_height.get();
        (iw / 2.0, ih / 2.0)
    });

    let radar_radius = Memo::new(move |_| {
        let iw = inner_width.get();
        let ih = inner_height.get();
        (iw.min(ih)) / 2.0 * 0.75
    });

    // Precompute pixel-space vertices for each series (for tooltip hit-testing)
    let series_tooltip: Memo<Vec<RadarTooltipSeries>> = Memo::new(move |_| {
        let (cx, cy) = radar_center.get();
        let r = radar_radius.get();
        let max_v = max_val.get();
        let series_data = data.get();
        let th = theme.get();
        series_data
            .iter()
            .enumerate()
            .map(|(si, s)| {
                let color = th.palette[si % th.palette.len()].clone();
                let vertices: Vec<(f64, f64)> = (0..n_axes)
                    .map(|i| {
                        let val = s.values.get(i).copied().unwrap_or(0.0);
                        let rv = (val / max_v).clamp(0.0, 1.0) * r;
                        let angle = spoke_angle(i);
                        (cx + rv * angle.cos(), cy + rv * angle.sin())
                    })
                    .collect();
                RadarTooltipSeries {
                    name: s.name.clone(),
                    color,
                    vertices,
                    values: s.values.clone(),
                }
            })
            .collect()
    });

    let show_legend = Memo::new(move |_| {
        config
            .get()
            .show_legend
            .unwrap_or_else(|| legend_items.get().len() > 1)
    });

    view! {
        <div
            class="radar-chart"
            style=move || {
                format!(
                    "width: 100%; height: 100%; display: flex; flex-direction: column; background-color: {};",
                    theme.get().background_color,
                )
            }
        >
            {move || {
                final_title
                    .get()
                    .map(|t| {
                        let th = theme.get();
                        view! {
                            <h3 style=format!(
                                "text-align: center; margin: 0; padding: 0.5rem; font-size: {}px; font-family: {}; color: {};",
                                th.font_size + 2.0,
                                th.font_family,
                                th.text_color,
                            )>{t}</h3>
                        }
                    })
            }}
            <div node_ref=container_ref style="flex: 1; position: relative; min-height: 0;">
                <svg
                    role="img"
                    aria-label="Radar chart"
                    viewBox=move || format!("0 0 {} {}", chart_width.get(), chart_height.get())
                    style="width: 100%; height: 100%; display: block;"
                >
                    <g transform=move || {
                        format!("translate({}, {})", margin.get().left, margin.get().top)
                    }>
                        {move || {
                            let iw = inner_width.get();
                            let ih = inner_height.get();
                            let cx = iw / 2.0;
                            let cy = ih / 2.0;
                            let radius = (iw.min(ih)) / 2.0 * 0.75;
                            let max_v = max_val.get();
                            let th = theme.get();
                            let series = data.get();
                            let grid_color = &th.grid.color;
                            let axis_color = &th.axis_color;
                            let text_color = &th.text_color;
                            let grid_polys: Vec<_> = (1..=grid_levels)
                                .map(|lvl| {
                                    let r = radius * lvl as f64 / grid_levels as f64;
                                    let pts: Vec<(f64, f64)> = (0..n_axes)
                                        .map(|i| polar_to_cart(cx, cy, r, spoke_angle(i)))
                                        .collect();
                                    let label = format!(
                                        "{:.0}",
                                        max_v * lvl as f64 / grid_levels as f64,
                                    );
                                    let label_x = cx + 4.0;
                                    let label_y = cy - r + 2.0;
                                    let pts_str = polygon_points(&pts);
                                    let gc = grid_color.clone();
                                    let tc = text_color.clone();
                                    let fs = th.axis_font_size;

                                    // ── Grid polygons ──────────────────────────────
                                    view! {
                                        <g>
                                            <polygon
                                                points=pts_str
                                                fill="none"
                                                stroke=gc
                                                stroke-width=0.8
                                                opacity=0.6
                                            />
                                            <text
                                                x=format!("{label_x:.2}")
                                                y=format!("{label_y:.2}")
                                                font-size=fs - 1.0
                                                fill=tc
                                                opacity=0.7
                                            >
                                                {label}
                                            </text>
                                        </g>
                                    }
                                })
                                .collect();
                            let spokes: Vec<_> = (0..n_axes)
                                .map(|i| {
                                    let angle = spoke_angle(i);
                                    let (ex, ey) = polar_to_cart(cx, cy, radius, angle);
                                    let label_r = radius + 18.0;
                                    let (lx, ly) = polar_to_cart(cx, cy, label_r, angle);
                                    let anchor = if lx < cx - 5.0 {
                                        "end"
                                    } else if lx > cx + 5.0 {
                                        "start"
                                    } else {
                                        "middle"
                                    };
                                    let label = axes_clone.get(i).cloned().unwrap_or_default();
                                    let ac = axis_color.clone();
                                    let tc = text_color.clone();
                                    let fs = th.axis_font_size;

                                    // ── Spokes and axis labels ─────────────────────
                                    // Label slightly outside the grid
                                    view! {
                                        <g>
                                            <line
                                                x1=format!("{cx:.2}")
                                                y1=format!("{cy:.2}")
                                                x2=format!("{ex:.2}")
                                                y2=format!("{ey:.2}")
                                                stroke=ac
                                                stroke-width=0.8
                                                opacity=0.5
                                            />
                                            <text
                                                x=format!("{lx:.2}")
                                                y=format!("{ly:.2}")
                                                text-anchor=anchor
                                                dominant-baseline="middle"
                                                font-size=fs
                                                fill=tc
                                            >
                                                {label}
                                            </text>
                                        </g>
                                    }
                                })
                                .collect();
                            let data_polys: Vec<_> = series
                                .iter()
                                .enumerate()
                                .map(|(si, s)| {
                                    let color = th.palette[si % th.palette.len()].clone();
                                    let pts: Vec<(f64, f64)> = (0..n_axes)
                                        .map(|i| {
                                            let val = s.values.get(i).copied().unwrap_or(0.0);
                                            let r = (val / max_v).clamp(0.0, 1.0) * radius;
                                            polar_to_cart(cx, cy, r, spoke_angle(i))
                                        })
                                        .collect();
                                    let pts_str = polygon_points(&pts);

                                    // ── Data polygons ──────────────────────────────
                                    view! {
                                        <polygon
                                            points=pts_str
                                            fill=format!("{}44", color)
                                            stroke=color.clone()
                                            stroke-width=2
                                            stroke-linejoin="round"
                                        />
                                    }
                                })
                                .collect();

                            view! { <g>{grid_polys} {spokes} {data_polys}</g> }
                        }} // Tooltip overlay (last = captures mouse events above chart content)
                        <RadarTooltip
                            series=series_tooltip
                            axes=axes_for_tooltip
                            radius=radar_radius
                            inner_width=inner_width
                            inner_height=inner_height
                            margin=margin
                        /> // SVG Legend overlay (must be last to render on top)
                        {move || {
                            show_legend
                                .get()
                                .then(|| {
                                    let text_color = theme.get().text_color;
                                    let position = if legend_outside.get() {
                                        LegendPosition::ExternalRight
                                    } else {
                                        LegendPosition::TopRight
                                    };
                                    view! {
                                        <Legend
                                            items=legend_items
                                            position=position
                                            inner_width=inner_width
                                            inner_height=inner_height
                                            text_color=text_color
                                        />
                                    }
                                })
                        }}
                    </g>
                </svg>
            </div>
        </div>
    }
}

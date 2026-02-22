/// BarChart component with vertical/horizontal, grouped and stacked modes
use crate::components::svg::axis::{Axis, AxisOrientation};
use crate::components::svg::bar_tooltip::{BarTooltip, BarTooltipSeries};
use crate::components::svg::grid::Grid;
use crate::components::svg::legend::{estimate_legend_width, Legend, LegendItem, LegendPosition};
use crate::hooks::use_container_size;
use leptos::prelude::*;
use lodviz_core::algorithms::stack::stack_series;
use lodviz_core::core::a11y;
use lodviz_core::core::data::{BarDataset, BarSeries};
use lodviz_core::core::mark::Mark;
use lodviz_core::core::scale::{BandScale, LinearScale, Scale};
use lodviz_core::core::theme::{ChartConfig, GridStyle};

/// Bar orientation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BarOrientation {
    #[default]
    /// Vertical bars (columns)
    Vertical,
    /// Horizontal bars
    Horizontal,
}

/// Bar layout mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BarMode {
    #[default]
    /// Side-by-side grouped bars for multiple series
    Grouped,
    /// Stacked bars for cumulative series values
    Stacked,
}

/// BarChart component for rendering bar charts
///
/// Features:
/// - Vertical and horizontal orientations
/// - Grouped and stacked bar modes
/// - Multi-series support
/// - Legend with click-to-toggle
/// - Responsive SVG rendering
#[component]
pub fn BarChart(
    /// Bar dataset with categories and series
    data: Signal<BarDataset>,
    /// Width (optional)
    #[prop(optional)]
    width: Option<u32>,
    /// Height (optional)
    #[prop(optional)]
    height: Option<u32>,
    /// Chart title
    #[prop(optional)]
    title: Option<String>,
    /// Show grid
    #[prop(default = true)]
    show_grid: bool,
    /// Bar orientation
    #[prop(default = BarOrientation::Vertical)]
    orientation: BarOrientation,
    /// Bar mode (grouped or stacked)
    #[prop(default = BarMode::Grouped)]
    mode: BarMode,
    /// X axis label
    #[prop(optional, into)]
    x_label: Option<String>,
    /// Y axis label
    #[prop(optional, into)]
    y_label: Option<String>,
    /// Chart configuration
    #[prop(default = Signal::derive(|| ChartConfig::default()), into)]
    config: Signal<ChartConfig>,
) -> impl IntoView {
    let theme = Memo::new(move |_| config.get().theme.unwrap_or_default());
    let (container_width, container_height, container_ref) = use_container_size();

    let chart_width = Memo::new(move |_| {
        let measured = container_width.get();
        if measured > 0.0 {
            return measured as u32;
        }
        config.get().width.or(width).unwrap_or(800)
    });

    let chart_height = Memo::new(move |_| {
        let measured = container_height.get();
        if measured > 0.0 {
            return measured as u32;
        }
        config.get().height.or(height).unwrap_or(400)
    });

    // Series visibility (defined early — needed by legend_items before margin)
    let (series_visibility, set_series_visibility) = signal(Vec::<bool>::new());

    Effect::new(move |_| {
        let n = data.get().series.len();
        let current = series_visibility.get_untracked();
        if current.len() != n {
            set_series_visibility.set(vec![true; n]);
        }
    });

    // Legend items — defined early so margin can adapt when legend_outside is enabled
    let legend_items = Signal::derive(move || {
        let d = data.get();
        let vis = series_visibility.get();
        let th = theme.get();
        d.series
            .iter()
            .enumerate()
            .map(|(i, s)| LegendItem {
                name: s.name.clone(),
                color: th.palette[i % th.palette.len()].clone(),
                visible: vis.get(i).copied().unwrap_or(true),
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

    let final_title = Memo::new(move |_| config.get().title.or(title.clone()));
    let grid_style = Memo::new(move |_| {
        config.get().grid.unwrap_or_else(|| {
            let th = theme.get();
            if show_grid {
                th.grid.clone()
            } else {
                GridStyle {
                    show_x: false,
                    show_y: false,
                    ..th.grid.clone()
                }
            }
        })
    });

    // Band scale for categories (always on primary axis)
    let band_scale = Memo::new(move |_| {
        let d = data.get();
        let range = match orientation {
            BarOrientation::Vertical => (0.0, inner_width.get()),
            BarOrientation::Horizontal => (0.0, inner_height.get()),
        };
        BandScale::new(d.categories.clone(), range, 0.2)
    });

    // Value scale (linear) computed from all visible series
    let value_scale = Memo::new(move |_| {
        let d = data.get();
        let vis = series_visibility.get();
        let range_size = match orientation {
            BarOrientation::Vertical => inner_height.get(),
            BarOrientation::Horizontal => inner_width.get(),
        };

        let max_val = match mode {
            BarMode::Grouped => d
                .series
                .iter()
                .enumerate()
                .filter(|(i, _)| vis.get(*i).copied().unwrap_or(true))
                .flat_map(|(_, s)| s.values.iter())
                .fold(0.0_f64, |acc, &v| acc.max(v)),
            BarMode::Stacked => {
                let n_cat = d.categories.len();
                (0..n_cat)
                    .map(|ci| {
                        d.series
                            .iter()
                            .enumerate()
                            .filter(|(i, _)| vis.get(*i).copied().unwrap_or(true))
                            .map(|(_, s)| s.values.get(ci).copied().unwrap_or(0.0))
                            .sum::<f64>()
                    })
                    .fold(0.0_f64, f64::max)
            }
        };

        let max_val = if max_val <= 0.0 { 1.0 } else { max_val * 1.1 }; // 10% padding

        match orientation {
            BarOrientation::Vertical => LinearScale::new((0.0, max_val), (range_size, 0.0)),
            BarOrientation::Horizontal => LinearScale::new((0.0, max_val), (0.0, range_size)),
        }
    });

    let x_tick_count = Memo::new(move |_| (inner_width.get() / 100.0).max(2.0) as usize);
    let y_tick_count = Memo::new(move |_| (inner_height.get() / 50.0).max(2.0) as usize);

    // A11y
    let chart_description = Memo::new(move |_| {
        let d = data.get();
        let total = d.categories.len() * d.series.len();
        let mut desc = a11y::generate_chart_description(Mark::Bar, total, None, None);
        if d.series.len() > 1 {
            desc.push_str(&format!(" {} series: ", d.series.len()));
            let names: Vec<_> = d.series.iter().map(|s| s.name.as_str()).collect();
            desc.push_str(&names.join(", "));
            desc.push('.');
        }
        desc
    });

    let aria_label =
        Memo::new(move |_| final_title.get().unwrap_or_else(|| "Bar chart".to_string()));

    // Tooltip data
    let tooltip_series_info = Memo::new(move |_| {
        let d = data.get();
        let vis = series_visibility.get();
        let th = theme.get();
        d.series
            .iter()
            .enumerate()
            .map(|(i, s)| BarTooltipSeries {
                name: s.name.clone(),
                values: s.values.clone(),
                color: th.palette[i % th.palette.len()].clone(),
                visible: vis.get(i).copied().unwrap_or(true),
            })
            .collect::<Vec<_>>()
    });
    let tooltip_categories = Memo::new(move |_| data.get().categories.clone());

    let on_legend_toggle = Callback::new(move |idx: usize| {
        let mut vis = series_visibility.get();
        if let Some(v) = vis.get_mut(idx) {
            *v = !*v;
        }
        set_series_visibility.set(vis);
    });

    let show_legend = Memo::new(move |_| {
        config
            .get()
            .show_legend
            .unwrap_or_else(|| legend_items.get().len() > 1)
    });

    let x_label_clone = x_label.clone();
    let y_label_clone = y_label.clone();

    view! {
        <div
            class="bar-chart"
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
                    aria-label=move || aria_label.get()
                    tabindex="0"
                    viewBox=move || format!("0 0 {} {}", chart_width.get(), chart_height.get())
                    style="width: 100%; height: 100%; display: block; outline: none; will-change: transform;"
                >
                    <title>{move || aria_label.get()}</title>
                    <desc>{move || chart_description.get()}</desc>
                    <g transform=move || {
                        format!("translate({}, {})", margin.get().left, margin.get().top)
                    }>
                        // Grid
                        {move || {
                            let gs = grid_style.get();
                            if gs.show_x || gs.show_y {
                                let vs = value_scale.get();
                                let (x_s, y_s) = match orientation {
                                    BarOrientation::Vertical => {
                                        (
                                            LinearScale::new(
                                                (0.0, inner_width.get()),
                                                (0.0, inner_width.get()),
                                            ),
                                            vs,
                                        )
                                    }
                                    BarOrientation::Horizontal => {
                                        (
                                            vs,
                                            LinearScale::new(
                                                (0.0, inner_height.get()),
                                                (0.0, inner_height.get()),
                                            ),
                                        )
                                    }
                                };
                                Some(
                                    view! {
                                        <Grid
                                            x_scale=x_s
                                            y_scale=y_s
                                            tick_count=x_tick_count.get()
                                            width=inner_width.get()
                                            height=inner_height.get()
                                            style=gs
                                        />
                                    },
                                )
                            } else {
                                None
                            }
                        }} // Bar rects
                        {move || {
                            let d = data.get();
                            let vis = series_visibility.get();
                            let bs = band_scale.get();
                            let vs = value_scale.get();
                            let th = theme.get();
                            let visible_series: Vec<(usize, &BarSeries)> = d
                                .series
                                .iter()
                                .enumerate()
                                .filter(|(i, _)| vis.get(*i).copied().unwrap_or(true))
                                .collect();
                            let n_visible = visible_series.len();
                            let mut bars: Vec<(String, String, String, String, String, String)> = vec![];
                            match mode {
                                BarMode::Grouped => {
                                    let sub_band_width = if n_visible > 0 {
                                        bs.band_width() / n_visible as f64
                                    } else {
                                        0.0
                                    };
                                    for (vi, (si, series)) in visible_series.iter().enumerate() {
                                        let color = th.palette[*si % th.palette.len()].clone();
                                        for (ci, &val) in series.values.iter().enumerate() {
                                            let cat = d
                                                .categories
                                                .get(ci)
                                                .map(|s| s.as_str())
                                                .unwrap_or("");
                                            let label = format!("{cat}: {} = {val:.1}", series.name);
                                            let (rx, ry, rw, rh) = match orientation {
                                                BarOrientation::Vertical => {
                                                    let x = bs.map_index(ci) + vi as f64 * sub_band_width;
                                                    let y = vs.map(val);
                                                    let h = vs.map(0.0) - y;
                                                    (x, y, sub_band_width, h.max(0.0))
                                                }
                                                BarOrientation::Horizontal => {
                                                    let y = bs.map_index(ci) + vi as f64 * sub_band_width;
                                                    let w = vs.map(val);
                                                    (0.0, y, w.max(0.0), sub_band_width)
                                                }
                                            };
                                            bars.push((
                                                format!("{rx:.2}"),
                                                format!("{ry:.2}"),
                                                format!("{rw:.2}"),
                                                format!("{rh:.2}"),
                                                color.clone(),
                                                label,
                                            ));
                                        }
                                    }
                                }
                                BarMode::Stacked => {
                                    let series_vals: Vec<Vec<f64>> = visible_series
                                        .iter()
                                        .map(|(_, s)| s.values.clone())
                                        .collect();
                                    let stacked = stack_series(&series_vals);
                                    let bw = bs.band_width();
                                    for (stack_i, stacked_s) in stacked.iter().enumerate() {
                                        let (si, _) = visible_series[stack_i];
                                        let color = th.palette[si % th.palette.len()].clone();
                                        let series_name = &d.series[si].name;
                                        for (ci, sv) in stacked_s.values.iter().enumerate() {
                                            let cat = d
                                                .categories
                                                .get(ci)
                                                .map(|s| s.as_str())
                                                .unwrap_or("");
                                            let val = sv.y1 - sv.y0;
                                            let label = format!("{cat}: {series_name} = {val:.1}");
                                            let (rx, ry, rw, rh) = match orientation {
                                                BarOrientation::Vertical => {
                                                    let x = bs.map_index(ci);
                                                    let y_top = vs.map(sv.y1);
                                                    let y_bot = vs.map(sv.y0);
                                                    (x, y_top, bw, (y_bot - y_top).max(0.0))
                                                }
                                                BarOrientation::Horizontal => {
                                                    let y = bs.map_index(ci);
                                                    let x_start = vs.map(sv.y0);
                                                    let x_end = vs.map(sv.y1);
                                                    (x_start, y, (x_end - x_start).max(0.0), bw)
                                                }
                                            };
                                            bars.push((
                                                format!("{rx:.2}"),
                                                format!("{ry:.2}"),
                                                format!("{rw:.2}"),
                                                format!("{rh:.2}"),
                                                color.clone(),
                                                label,
                                            ));
                                        }
                                    }
                                }
                            }
                            bars.into_iter()
                                .map(|(rx, ry, rw, rh, color, label)| {
                                    // Pre-compute all bar rectangles as (x, y, w, h, color, label)

                                    view! {
                                        <rect
                                            x=rx
                                            y=ry
                                            width=rw
                                            height=rh
                                            fill=color
                                            aria-label=label
                                        />
                                    }
                                })
                                .collect_view()
                        }} // Category labels on the categorical axis
                        {move || {
                            let d = data.get();
                            let bs = band_scale.get();
                            let th = theme.get();
                            d.categories
                                .iter()
                                .enumerate()
                                .map(|(i, cat)| {
                                    {
                                        let (tx, ty, anchor) = match orientation {
                                            BarOrientation::Vertical => {
                                                let x = bs.map_index_center(i);
                                                let y = inner_height.get() + 16.0;
                                                (format!("{x:.2}"), format!("{y:.2}"), "middle".to_string())
                                            }
                                            BarOrientation::Horizontal => {
                                                let y = bs.map_index_center(i);
                                                (
                                                    "-8".to_string(),
                                                    format!("{:.2}", y + 4.0),
                                                    "end".to_string(),
                                                )
                                            }
                                        };
                                        view! {
                                            <text
                                                x=tx
                                                y=ty
                                                text-anchor=anchor
                                                font-size=th.axis_font_size
                                                fill=th.axis_color.clone()
                                            >
                                                {cat.clone()}
                                            </text>
                                        }
                                    }
                                })
                                .collect_view()
                        }} // Value axis
                        {move || {
                            let vs = value_scale.get();
                            match orientation {
                                BarOrientation::Vertical => {
                                    view! {
                                        <Axis
                                            orientation=AxisOrientation::Left
                                            scale=vs
                                            tick_count=y_tick_count.get()
                                            _dimension=inner_height.get()
                                            stroke=theme.get().axis_color
                                            font_size=theme.get().axis_font_size
                                            label=y_label_clone.clone()
                                        />
                                    }
                                        .into_any()
                                }
                                BarOrientation::Horizontal => {
                                    view! {
                                        <g transform=format!(
                                            "translate(0, {})",
                                            inner_height.get(),
                                        )>
                                            <Axis
                                                orientation=AxisOrientation::Bottom
                                                scale=vs
                                                tick_count=x_tick_count.get()
                                                _dimension=inner_width.get()
                                                stroke=theme.get().axis_color
                                                font_size=theme.get().axis_font_size
                                                label=x_label_clone.clone()
                                            />
                                        </g>
                                    }
                                        .into_any()
                                }
                            }
                        }} // Tooltip (must be last for z-order)
                        <BarTooltip
                            categories=tooltip_categories
                            series_info=tooltip_series_info
                            band_scale=band_scale
                            value_scale=value_scale
                            inner_width=inner_width
                            inner_height=inner_height
                            orientation=orientation
                            mode=mode
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
                                            on_toggle=on_legend_toggle
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

/// Heatmap chart component
use crate::components::svg::colorbar::ColorBar;
use crate::components::svg::heatmap_tooltip::HeatmapTooltip;
use crate::hooks::use_container_size;
use leptos::prelude::*;
use lodviz_core::core::color_map::{ColorMap, SequentialColorMap};
use lodviz_core::core::data::GridData;
use lodviz_core::core::theme::ChartConfig;

/// HeatmapChart: renders a 2-D grid as colored rectangles using a continuous ColorMap.
///
/// Features:
/// - Configurable color map (any `ColorMap` variant)
/// - Optional cell value labels
/// - Optional ColorBar legend
/// - Hover tooltip (row, col, value)
/// - Responsive via `use_container_size()`
#[component]
pub fn HeatmapChart(
    /// 2-D grid data (rows × columns)
    data: Signal<GridData>,
    /// Color map for encoding values as colors
    #[prop(default = ColorMap::Sequential(SequentialColorMap::Viridis))]
    color_map: ColorMap,
    /// Show numeric value labels inside each cell
    #[prop(default = false)]
    show_values: bool,
    /// Show a vertical color bar on the right
    #[prop(default = true)]
    show_colorbar: bool,
    /// Chart configuration
    #[prop(default = Signal::derive(|| ChartConfig::default()), into)]
    config: Signal<ChartConfig>,
    /// Fixed width override
    #[prop(optional)]
    width: Option<u32>,
    /// Fixed height override
    #[prop(optional)]
    height: Option<u32>,
) -> impl IntoView {
    let theme = Memo::new(move |_| config.get().theme.unwrap_or_default());
    let (container_width, container_height, container_ref) = use_container_size();

    let chart_width = Memo::new(move |_| {
        let m = container_width.get();
        if m > 0.0 {
            return m as u32;
        }
        config.get().width.or(width).unwrap_or(600)
    });
    let chart_height = Memo::new(move |_| {
        let m = container_height.get();
        if m > 0.0 {
            return m as u32;
        }
        config.get().height.or(height).unwrap_or(400)
    });

    let final_title = Memo::new(move |_| config.get().title);

    // Colorbar width in margin right
    let colorbar_w = if show_colorbar { 60.0 } else { 0.0 };

    let margin_top = 40.0_f64;
    let margin_bottom = 60.0_f64;
    let margin_left = 70.0_f64;
    let margin_right = colorbar_w + 20.0;

    let inner_width =
        Memo::new(move |_| (chart_width.get() as f64 - margin_left - margin_right).max(10.0));
    let inner_height =
        Memo::new(move |_| (chart_height.get() as f64 - margin_top - margin_bottom).max(10.0));

    // Derived grid stats
    let grid_stats = Memo::new(move |_| {
        let g = data.get();
        let nrows = g.values.len();
        let ncols = g.values.first().map(|r| r.len()).unwrap_or(0);
        let min = g.min();
        let max_v = g.max();
        let range = (max_v - min).max(1e-12);
        (nrows, ncols, min, max_v, range)
    });

    // Hover state
    let (hover_row, set_hover_row) = signal(None::<usize>);
    let (hover_col, set_hover_col) = signal(None::<usize>);
    let (hover_x, set_hover_x) = signal(0.0_f64);
    let (hover_y, set_hover_y) = signal(0.0_f64);

    let hover_row_label = Signal::derive(move || {
        let g = data.get();
        hover_row.get().and_then(|r| {
            g.row_labels
                .as_ref()
                .and_then(|labels| labels.get(r))
                .cloned()
                .or_else(|| Some(format!("Row {}", r + 1)))
        })
    });
    let hover_col_label = Signal::derive(move || {
        let g = data.get();
        hover_col.get().and_then(|c| {
            g.col_labels
                .as_ref()
                .and_then(|labels| labels.get(c))
                .cloned()
                .or_else(|| Some(format!("Col {}", c + 1)))
        })
    });
    let hover_value = Signal::derive(move || {
        let g = data.get();
        hover_row.get().and_then(|r| {
            hover_col
                .get()
                .and_then(|c| g.values.get(r)?.get(c).copied())
        })
    });

    let color_map_clone = color_map.clone();
    let color_map_colorbar = color_map.clone();
    let clip_id = format!("heatmap-clip-{}", uuid::Uuid::new_v4().simple());

    let iw_signal = Signal::derive(move || inner_width.get());
    let ih_signal = Signal::derive(move || inner_height.get());

    let a11y_title_id = format!("chart-title-{}", uuid::Uuid::new_v4().as_simple());
    let a11y_desc_id = format!("chart-desc-{}", uuid::Uuid::new_v4().as_simple());
    let a11y_labelledby = format!("{} {}", a11y_title_id, a11y_desc_id);

    view! {
        <div
            class="heatmap-chart"
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
                                "text-align: center; margin: 0; padding-top: {}px; padding-bottom: {}px; font-size: {}px; font-family: {}; color: {}; font-weight: {};",
                                th.title_padding_top,
                                th.title_padding_bottom,
                                th.title_font_size,
                                th.font_family,
                                th.text_color,
                                th.title_font_weight,
                            )>{t}</h3>
                        }
                    })
            }}

            <div node_ref=container_ref style="flex: 1; position: relative; min-height: 0;">
                <svg
                    role="img"
                    aria-labelledby=a11y_labelledby
                    viewBox=move || format!("0 0 {} {}", chart_width.get(), chart_height.get())
                    style="width: 100%; height: 100%; display: block;"
                >
                    <title id=a11y_title_id>{move || final_title.get().unwrap_or("Heatmap".to_string())}</title>
                    <desc id=a11y_desc_id>"Heatmap showing data values encoded as a color gradient across a two-dimensional grid."</desc>

                    <g transform=move || format!("translate({margin_left}, {margin_top})")>
                        <defs>
                            <clipPath id=clip_id.clone()>
                                <rect
                                    x="0"
                                    y="0"
                                    width=move || inner_width.get()
                                    height=move || inner_height.get()
                                />
                            </clipPath>
                        </defs>

                        // Grid cells
                        {move || {
                            let g = data.get();
                            let (nrows, ncols, min, _, range) = grid_stats.get();
                            if nrows == 0 || ncols == 0 {
                                return vec![].into_iter().collect_view();
                            }
                            let iw = inner_width.get();
                            let ih = inner_height.get();
                            let cell_w = iw / ncols as f64;
                            let cell_h = ih / nrows as f64;
                            let th = theme.get();
                            let cm = color_map_clone.clone();
                            g.values
                                .iter()
                                .enumerate()
                                .flat_map(|(row, row_vals)| {
                                    let cm_row = cm.clone();
                                    let tc_row = th.text_color.clone();
                                    row_vals
                                        .iter()
                                        .enumerate()
                                        .map(move |(col, &val)| {
                                            let t = (val - min) / range;
                                            let fill = cm_row.map(t);
                                            let x = col as f64 * cell_w;
                                            let y = row as f64 * cell_h;
                                            let label = if show_values {
                                                Some(format!("{val:.2}"))
                                            } else {
                                                None
                                            };
                                            let font_size = (cell_h * 0.35).clamp(7.0, 12.0);
                                            let tc = tc_row.clone();

                                            view! {
                                                <g
                                                    style="cursor: default;"
                                                    on:mousemove=move |ev| {
                                                        set_hover_row.set(Some(row));
                                                        set_hover_col.set(Some(col));
                                                        set_hover_x.set(ev.offset_x() as f64 - margin_left);
                                                        set_hover_y.set(ev.offset_y() as f64 - margin_top);
                                                    }
                                                    on:mouseleave=move |_| {
                                                        set_hover_row.set(None);
                                                        set_hover_col.set(None);
                                                    }
                                                >
                                                    <rect
                                                        x=format!("{x:.2}")
                                                        y=format!("{y:.2}")
                                                        width=format!("{cell_w:.2}")
                                                        height=format!("{cell_h:.2}")
                                                        fill=fill
                                                        stroke="none"
                                                    />
                                                    {label
                                                        .map(|lbl| {
                                                            view! {
                                                                <text
                                                                    x=format!("{:.2}", x + cell_w / 2.0)
                                                                    y=format!("{:.2}", y + cell_h / 2.0 + font_size * 0.35)
                                                                    text-anchor="middle"
                                                                    font-size=font_size
                                                                    fill=tc.clone()
                                                                    pointer-events="none"
                                                                >
                                                                    {lbl}
                                                                </text>
                                                            }
                                                        })}
                                                </g>
                                            }
                                        })
                                })
                                .collect_view()
                        }}

                        // Row axis labels (Y)
                        {move || {
                            let g = data.get();
                            let (nrows, _, _, _, _) = grid_stats.get();
                            if nrows == 0 {
                                return vec![].into_iter().collect_view();
                            }
                            let ih = inner_height.get();
                            let cell_h = ih / nrows as f64;
                            let th = theme.get();
                            (0..nrows)
                                .map(|row| {
                                    let label = g
                                        .row_labels
                                        .as_ref()
                                        .and_then(|l| l.get(row))
                                        .cloned()
                                        .unwrap_or_else(|| format!("{}", row + 1));
                                    let y = row as f64 * cell_h + cell_h / 2.0;
                                    view! {
                                        <text
                                            x="-6"
                                            y=format!("{:.2}", y + th.axis_font_size * 0.35)
                                            text-anchor="end"
                                            font-size=th.axis_font_size
                                            fill=th.text_color.clone()
                                        >
                                            {label}
                                        </text>
                                    }
                                })
                                .collect_view()
                        }}

                        // Column axis labels (X)
                        {move || {
                            let g = data.get();
                            let (_, ncols, _, _, _) = grid_stats.get();
                            if ncols == 0 {
                                return vec![].into_iter().collect_view();
                            }
                            let iw = inner_width.get();
                            let ih = inner_height.get();
                            let cell_w = iw / ncols as f64;
                            let th = theme.get();
                            (0..ncols)
                                .map(|col| {
                                    let label = g
                                        .col_labels
                                        .as_ref()
                                        .and_then(|l| l.get(col))
                                        .cloned()
                                        .unwrap_or_else(|| format!("{}", col + 1));
                                    let x = col as f64 * cell_w + cell_w / 2.0;
                                    view! {
                                        <text
                                            x=format!("{x:.2}")
                                            y=format!("{:.2}", ih + 16.0)
                                            text-anchor="middle"
                                            font-size=th.axis_font_size
                                            fill=th.text_color.clone()
                                        >
                                            {label}
                                        </text>
                                    }
                                })
                                .collect_view()
                        }}

                        // ColorBar
                        {move || {
                            if !show_colorbar {
                                return None;
                            }
                            let (_, _, min, max_v, _) = grid_stats.get();
                            let ih = inner_height.get();
                            let iw = inner_width.get();
                            let th = theme.get();
                            Some(
                                view! {
                                    <g transform=format!("translate({:.2}, 0)", iw + 10.0)>
                                        <ColorBar
                                            color_map=color_map_colorbar.clone()
                                            min_value=min
                                            max_value=max_v
                                            bar_width=14.0
                                            height=ih
                                            tick_count=5
                                            text_color=th.text_color.clone()
                                            font_size=th.axis_font_size
                                        />
                                    </g>
                                },
                            )
                        }}

                        // Tooltip
                        <HeatmapTooltip
                            row_label=hover_row_label
                            col_label=hover_col_label
                            value=hover_value
                            x=Signal::derive(move || hover_x.get())
                            y=Signal::derive(move || hover_y.get())
                            inner_width=iw_signal
                            inner_height=ih_signal
                            text_color="white".to_string()
                        />
                    </g>
                </svg>
            </div>
        </div>
    }
}

/// Waterfall chart component
use crate::components::svg::axis::{Axis, AxisOrientation};
use crate::components::svg::grid::Grid;
use crate::components::svg::waterfall_tooltip::{WaterfallTooltip, WaterfallTooltipEntry};
use crate::hooks::use_container_size;
use leptos::prelude::*;
use lodviz_core::core::data::{WaterfallBar, WaterfallKind};
use lodviz_core::core::scale::{BandScale, LinearScale, Scale};
use lodviz_core::core::theme::ChartConfig;

/// Waterfall chart — cumulative bar chart with connector lines
///
/// Renders Start (base), Delta (incremental), and Total bars with
/// horizontal connector lines between consecutive bars.
#[component]
pub fn WaterfallChart(
    /// Bars in display order
    data: Signal<Vec<WaterfallBar>>,
    /// Chart configuration
    #[prop(default = Signal::derive(|| ChartConfig::default()), into)]
    config: Signal<ChartConfig>,
    /// Fixed width
    #[prop(optional)]
    width: Option<u32>,
    /// Fixed height
    #[prop(optional)]
    height: Option<u32>,
    /// Y axis label
    #[prop(optional, into)]
    y_label: Option<String>,
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

    let margin = Memo::new(move |_| config.get().margin.unwrap_or_default());
    let inner_width =
        Memo::new(move |_| chart_width.get() as f64 - margin.get().left - margin.get().right);
    let inner_height =
        Memo::new(move |_| chart_height.get() as f64 - margin.get().top - margin.get().bottom);

    let final_title = Memo::new(move |_| config.get().title.clone());

    /// A computed bar with baseline for rendering
    #[derive(Clone, PartialEq)]
    struct BarLayout {
        label: String,
        baseline: f64,
        top: f64,
        color: &'static str,
        /// Pixel-y of the top edge of this bar (for connector lines)
        connector_y: f64,
    }

    let computed = Memo::new(move |_| {
        let bars = data.get();
        let mut layouts: Vec<BarLayout> = Vec::with_capacity(bars.len());
        let mut running = 0.0_f64;

        for bar in &bars {
            match bar.kind {
                WaterfallKind::Start => {
                    running = bar.value;
                    layouts.push(BarLayout {
                        label: bar.label.clone(),
                        baseline: 0.0,
                        top: bar.value,
                        color: "#4CAF50",
                        connector_y: 0.0, // will be computed in SVG
                    });
                }
                WaterfallKind::Delta => {
                    let base = running;
                    running += bar.value;
                    let (bot, top_val) = if bar.value >= 0.0 {
                        (base, running)
                    } else {
                        (running, base)
                    };
                    let color = if bar.value >= 0.0 {
                        "#26a69a"
                    } else {
                        "#ef5350"
                    };
                    layouts.push(BarLayout {
                        label: bar.label.clone(),
                        baseline: bot,
                        top: top_val,
                        color,
                        connector_y: 0.0,
                    });
                }
                WaterfallKind::Total => {
                    layouts.push(BarLayout {
                        label: bar.label.clone(),
                        baseline: 0.0,
                        top: running,
                        color: "#607D8B",
                        connector_y: 0.0,
                    });
                }
            }
        }
        layouts
    });

    let y_scale = Memo::new(move |_| {
        let layouts = computed.get();
        let all: Vec<f64> = layouts.iter().flat_map(|l| [l.baseline, l.top]).collect();
        let y_min = all.iter().cloned().fold(0.0_f64, f64::min);
        let y_max = all.iter().cloned().fold(0.0_f64, f64::max);
        let pad = (y_max - y_min).abs() * 0.1;
        LinearScale::new((y_min - pad, y_max + pad), (inner_height.get(), 0.0))
    });

    let y_tick_count = Memo::new(move |_| (inner_height.get() / 50.0).max(2.0) as usize);
    let y_label_clone = y_label.clone();

    // Band scale memo for tooltip hit-testing
    let band_scale_memo = Memo::new(move |_| {
        let layouts = computed.get();
        BandScale::new(
            layouts.iter().map(|l| l.label.clone()).collect(),
            (0.0, inner_width.get()),
            0.15,
        )
    });

    // Tooltip entries memo (zip original data with computed layout)
    let entries_memo: Memo<Vec<WaterfallTooltipEntry>> = Memo::new(move |_| {
        let bars = data.get();
        let layouts = computed.get();
        bars.iter()
            .zip(layouts.iter())
            .map(|(bar, layout)| WaterfallTooltipEntry {
                label: bar.label.clone(),
                value: bar.value,
                running_total: layout.top,
                kind: bar.kind,
                color: layout.color.to_string(),
            })
            .collect()
    });

    view! {
        <div
            class="waterfall-chart"
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
                    aria-label="Waterfall chart"
                    viewBox=move || format!("0 0 {} {}", chart_width.get(), chart_height.get())
                    style="width: 100%; height: 100%; display: block;"
                >
                    <g transform=move || {
                        format!("translate({}, {})", margin.get().left, margin.get().top)
                    }>
                        // Grid (Y only)
                        {move || {
                            let ys = y_scale.get();
                            let dummy_xs = LinearScale::new(
                                (0.0, inner_width.get()),
                                (0.0, inner_width.get()),
                            );
                            view! {
                                <Grid
                                    x_scale=dummy_xs
                                    y_scale=ys
                                    tick_count=y_tick_count.get()
                                    width=inner_width.get()
                                    height=inner_height.get()
                                    style=theme.get().grid.clone()
                                />
                            }
                        }} // Zero line
                        {move || {
                            let ys = y_scale.get();
                            let y0 = ys.map(0.0);
                            let th = theme.get();
                            view! {
                                <line
                                    x1=0
                                    y1=format!("{y0:.2}")
                                    x2=inner_width.get()
                                    y2=format!("{y0:.2}")
                                    stroke=th.axis_color.clone()
                                    stroke-width=1
                                    opacity=0.4
                                />
                            }
                        }} // Bars + connectors + labels
                        {move || {
                            let layouts = computed.get();
                            let ys = y_scale.get();
                            let th = theme.get();
                            let iw = inner_width.get();
                            let x_band = BandScale::new(
                                layouts.iter().map(|l| l.label.clone()).collect(),
                                (0.0, iw),
                                0.15,
                            );
                            let bw = x_band.band_width();
                            layouts
                                .iter()
                                .enumerate()
                                .map(|(i, l)| {
                                    let bar_x = x_band.map_index(i);
                                    let cx_label = x_band.map_index_center(i);
                                    let y_top_px = ys.map(l.top);
                                    let y_bot_px = ys.map(l.baseline);
                                    let bar_y = y_top_px.min(y_bot_px);
                                    let bar_h = (y_top_px - y_bot_px).abs().max(1.0);
                                    let connector = if i + 1 < layouts.len() {
                                        let next_x = x_band.map_index(i + 1);
                                        let connector_y = ys.map(l.top);
                                        Some(

                                            // Connector to next bar
                                            view! {
                                                <line
                                                    x1=format!("{:.2}", bar_x + bw)
                                                    y1=format!("{connector_y:.2}")
                                                    x2=format!("{next_x:.2}")
                                                    y2=format!("{connector_y:.2}")
                                                    stroke="#999"
                                                    stroke-width=1
                                                    stroke-dasharray="3,2"
                                                />
                                            },
                                        )
                                    } else {
                                        None
                                    };
                                    let ty_label = inner_height.get() + 18.0;

                                    view! {
                                        <g>
                                            <rect
                                                x=format!("{bar_x:.2}")
                                                y=format!("{bar_y:.2}")
                                                width=format!("{bw:.2}")
                                                height=format!("{bar_h:.2}")
                                                fill=l.color
                                                opacity=0.85
                                            />
                                            {connector}
                                            <text
                                                x=format!("{cx_label:.2}")
                                                y=format!("{ty_label:.2}")
                                                text-anchor="middle"
                                                font-size=th.axis_font_size
                                                fill=th.axis_color.clone()
                                            >
                                                {l.label.clone()}
                                            </text>
                                        </g>
                                    }
                                })
                                .collect_view()
                        }} // Y axis
                        {move || {
                            view! {
                                <Axis
                                    orientation=AxisOrientation::Left
                                    scale=y_scale.get()
                                    tick_count=y_tick_count.get()
                                    _dimension=inner_height.get()
                                    stroke=theme.get().axis_color
                                    font_size=theme.get().axis_font_size
                                    label=y_label_clone.clone()
                                />
                            }
                        }} // Tooltip overlay (last = captures mouse events above chart content)
                        <WaterfallTooltip
                            entries=entries_memo
                            band_scale=band_scale_memo
                            inner_width=inner_width
                            inner_height=inner_height
                            margin=margin
                        />
                    </g>
                </svg>
            </div>
        </div>
    }
}

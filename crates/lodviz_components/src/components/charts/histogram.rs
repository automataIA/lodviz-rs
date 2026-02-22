/// Histogram chart component
use crate::components::svg::axis::{Axis, AxisOrientation};
use crate::components::svg::grid::Grid;
use crate::hooks::use_container_size;
use leptos::prelude::*;
use lodviz_core::algorithms::statistics::{histogram_bins, BinRule};
use lodviz_core::core::scale::{LinearScale, Scale};
use lodviz_core::core::theme::ChartConfig;

/// Histogram chart for continuous data
///
/// Bins raw `f64` values using the specified rule, then renders a bar for each bin.
/// Hovering a bar shows a tooltip with the bin range and count.
#[component]
pub fn Histogram(
    /// Raw data values
    data: Signal<Vec<f64>>,
    /// Binning strategy
    #[prop(default = BinRule::FreedmanDiaconis)]
    rule: BinRule,
    /// Chart configuration
    #[prop(default = Signal::derive(|| ChartConfig::default()), into)]
    config: Signal<ChartConfig>,
    /// Fixed width
    #[prop(optional)]
    width: Option<u32>,
    /// Fixed height
    #[prop(optional)]
    height: Option<u32>,
    /// X axis label
    #[prop(optional, into)]
    x_label: Option<String>,
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

    // Compute bins reactively
    let bins = Memo::new(move |_| histogram_bins(&data.get(), rule));

    let x_scale = Memo::new(move |_| {
        let bs = bins.get();
        let (x0, x1) = if bs.is_empty() {
            (0.0, 1.0)
        } else {
            (bs[0].x0, bs[bs.len() - 1].x1)
        };
        LinearScale::new((x0, x1), (0.0, inner_width.get()))
    });

    let y_scale = Memo::new(move |_| {
        let max_count = bins.get().iter().map(|b| b.count).max().unwrap_or(1);
        let max_val = (max_count as f64 * 1.1).max(1.0);
        LinearScale::new((0.0, max_val), (inner_height.get(), 0.0))
    });

    let x_tick_count = Memo::new(move |_| (inner_width.get() / 80.0).max(2.0) as usize);
    let y_tick_count = Memo::new(move |_| (inner_height.get() / 50.0).max(2.0) as usize);

    let (tooltip, set_tooltip) = signal(None::<(f64, f64, String)>);

    let x_label_clone = x_label.clone();
    let y_label_clone = y_label.clone();

    view! {
        <div
            class="histogram"
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
                    aria-label="Histogram"
                    viewBox=move || format!("0 0 {} {}", chart_width.get(), chart_height.get())
                    style="width: 100%; height: 100%; display: block;"
                    on:mouseleave=move |_| set_tooltip.set(None)
                >
                    <g transform=move || {
                        format!("translate({}, {})", margin.get().left, margin.get().top)
                    }>
                        // Grid
                        {move || {
                            let xs = x_scale.get();
                            let ys = y_scale.get();
                            view! {
                                <Grid
                                    x_scale=xs
                                    y_scale=ys
                                    tick_count=x_tick_count.get()
                                    width=inner_width.get()
                                    height=inner_height.get()
                                    style=theme.get().grid.clone()
                                />
                            }
                        }} // Bin bars
                        {move || {
                            let bs = bins.get();
                            let xs = x_scale.get();
                            let ys = y_scale.get();
                            let th = theme.get();
                            let color = th
                                .palette
                                .first()
                                .cloned()
                                .unwrap_or_else(|| "#5470c6".to_string());
                            let ih = inner_height.get();
                            bs.iter()
                                .map(|b| {
                                    let x0_px = xs.map(b.x0);
                                    let x1_px = xs.map(b.x1);
                                    let bar_w = (x1_px - x0_px - 1.0).max(0.5);
                                    let bar_h = ih - ys.map(b.count as f64);
                                    let bar_y = ys.map(b.count as f64);
                                    let tooltip_text = format!(
                                        "[{:.2}, {:.2}) — {} points",
                                        b.x0,
                                        b.x1,
                                        b.count,
                                    );
                                    let tt = tooltip_text.clone();
                                    let cx_pos = x0_px + bar_w / 2.0;
                                    let cy_pos = bar_y - 10.0_f64.max(bar_y - 15.0);

                                    view! {
                                        <rect
                                            x=format!("{x0_px:.2}")
                                            y=format!("{bar_y:.2}")
                                            width=format!("{bar_w:.2}")
                                            height=format!("{bar_h:.2}")
                                            fill=color.clone()
                                            opacity=0.8
                                            style="cursor: pointer;"
                                            on:mouseenter=move |_| {
                                                set_tooltip.set(Some((cx_pos, cy_pos, tt.clone())));
                                            }
                                        />
                                    }
                                })
                                .collect_view()
                        }} // Hover tooltip
                        {move || {
                            tooltip
                                .get()
                                .map(|(tx, ty, text)| {
                                    let th = theme.get();
                                    view! {
                                        <g>
                                            <rect
                                                x=format!("{:.2}", tx - 60.0)
                                                y=format!("{:.2}", ty - 24.0)
                                                width=120
                                                height=22
                                                rx=4
                                                fill="rgba(0,0,0,0.75)"
                                            />
                                            <text
                                                x=format!("{tx:.2}")
                                                y=format!("{:.2}", ty - 8.0)
                                                text-anchor="middle"
                                                font-size=th.axis_font_size
                                                fill="#ffffff"
                                            >
                                                {text}
                                            </text>
                                        </g>
                                    }
                                })
                        }} // X axis
                        <g transform=move || {
                            format!("translate(0, {})", inner_height.get())
                        }>
                            {move || {
                                view! {
                                    <Axis
                                        orientation=AxisOrientation::Bottom
                                        scale=x_scale.get()
                                        tick_count=x_tick_count.get()
                                        _dimension=inner_width.get()
                                        stroke=theme.get().axis_color
                                        font_size=theme.get().axis_font_size
                                        label=x_label_clone.clone()
                                    />
                                }
                            }}
                        // Y axis
                        </g>
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
                        }}
                    </g>
                </svg>
            </div>
        </div>
    }
}

/// Candlestick chart component for OHLC financial data
use crate::components::svg::axis::{Axis, AxisOrientation};
use crate::components::svg::candlestick_tooltip::CandlestickTooltip;
use crate::components::svg::grid::Grid;
use crate::hooks::use_container_size;
use leptos::prelude::*;
use lodviz_core::algorithms::m4::m4_downsample;
use lodviz_core::core::data::{DataPoint, OhlcBar};
use lodviz_core::core::scale::{LinearScale, Scale};
use lodviz_core::core::theme::ChartConfig;

/// Candlestick chart for OHLC financial data
///
/// Renders a wick line (high–low) and a body rect (open–close) per bar.
/// Bullish bars (close ≥ open) are green; bearish bars are red.
/// Applies M4 downsampling when more than 200 bars are provided.
#[component]
pub fn CandlestickChart(
    /// OHLC bars sorted by timestamp
    data: Signal<Vec<OhlcBar>>,
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

    // Visible bars after optional M4 downsampling on close prices
    let visible_bars = Memo::new(move |_| {
        let bars = data.get();
        if bars.len() <= 200 {
            return bars;
        }
        // Build DataPoint series (timestamp, close) for M4
        let pts: Vec<DataPoint> = bars
            .iter()
            .map(|b| DataPoint::new(b.timestamp, b.close))
            .collect();
        let n_pixels = (inner_width.get() as usize / 6).max(20);
        let downsampled = m4_downsample(&pts, n_pixels);
        // Keep only bars whose timestamps appear in the downsampled set
        let ts_set: std::collections::HashSet<u64> =
            downsampled.iter().map(|p| p.x.to_bits()).collect();
        bars.into_iter()
            .filter(|b| ts_set.contains(&b.timestamp.to_bits()))
            .collect()
    });

    let x_scale = Memo::new(move |_| {
        let bars = visible_bars.get();
        let (t_min, t_max) = if bars.is_empty() {
            (0.0, 1.0)
        } else {
            (
                bars.iter()
                    .map(|b| b.timestamp)
                    .fold(f64::INFINITY, f64::min),
                bars.iter()
                    .map(|b| b.timestamp)
                    .fold(f64::NEG_INFINITY, f64::max),
            )
        };
        let pad = (t_max - t_min) * 0.02;
        LinearScale::new((t_min - pad, t_max + pad), (0.0, inner_width.get()))
    });

    let y_scale = Memo::new(move |_| {
        let bars = visible_bars.get();
        let (y_min, y_max) = if bars.is_empty() {
            (0.0, 1.0)
        } else {
            (
                bars.iter().map(|b| b.low).fold(f64::INFINITY, f64::min) * 0.99,
                bars.iter()
                    .map(|b| b.high)
                    .fold(f64::NEG_INFINITY, f64::max)
                    * 1.01,
            )
        };
        LinearScale::new((y_min, y_max), (inner_height.get(), 0.0))
    });

    let x_tick_count = Memo::new(move |_| (inner_width.get() / 100.0).max(2.0) as usize);
    let y_tick_count = Memo::new(move |_| (inner_height.get() / 50.0).max(2.0) as usize);

    // Pixel width of each candlestick body (used for rendering AND tooltip highlight)
    let bar_pixel_width = Memo::new(move |_| {
        let n = visible_bars.get().len();
        if n > 1 {
            (inner_width.get() / n as f64 * 0.8).max(1.0)
        } else {
            10.0
        }
    });

    let x_label_clone = x_label.clone();
    let y_label_clone = y_label.clone();

    view! {
        <div
            class="candlestick-chart"
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
                    aria-label="Candlestick chart"
                    viewBox=move || format!("0 0 {} {}", chart_width.get(), chart_height.get())
                    style="width: 100%; height: 100%; display: block;"
                >
                    <g transform=move || {
                        format!("translate({}, {})", margin.get().left, margin.get().top)
                    }>
                        // Grid
                        {move || {
                            view! {
                                <Grid
                                    x_scale=x_scale.get()
                                    y_scale=y_scale.get()
                                    tick_count=x_tick_count.get()
                                    width=inner_width.get()
                                    height=inner_height.get()
                                    style=theme.get().grid.clone()
                                />
                            }
                        }} // Candles
                        {move || {
                            let bars = visible_bars.get();
                            let xs = x_scale.get();
                            let ys = y_scale.get();
                            let n = bars.len();
                            let bar_w = if n > 1 {
                                let step = inner_width.get() / n as f64;
                                (step * 0.8).max(1.0)
                            } else {
                                10.0
                            };
                            let half_bw = bar_w / 2.0;
                            bars.iter()
                                .map(|b| {
                                    let cx = xs.map(b.timestamp);
                                    let y_high = ys.map(b.high);
                                    let y_low = ys.map(b.low);
                                    let y_open = ys.map(b.open);
                                    let y_close = ys.map(b.close);
                                    let body_top = y_open.min(y_close);
                                    let body_h = (y_open - y_close).abs().max(1.0);
                                    let color = if b.is_bullish() { "#26a69a" } else { "#ef5350" };
                                    // Bar width as fraction of the step between adjacent bars

                                    view! {
                                        <g>
                                            // Wick
                                            <line
                                                x1=format!("{cx:.2}")
                                                y1=format!("{y_high:.2}")
                                                x2=format!("{cx:.2}")
                                                y2=format!("{y_low:.2}")
                                                stroke=color
                                                stroke-width=1
                                            />
                                            // Body
                                            <rect
                                                x=format!("{:.2}", cx - half_bw)
                                                y=format!("{body_top:.2}")
                                                width=format!("{bar_w:.2}")
                                                height=format!("{body_h:.2}")
                                                fill=color
                                            />
                                        </g>
                                    }
                                })
                                .collect_view()
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
                        }} // Tooltip overlay (last = captures mouse events above chart content)
                        <CandlestickTooltip
                            bars=visible_bars
                            x_scale=x_scale
                            y_scale=y_scale
                            bar_pixel_width=bar_pixel_width
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

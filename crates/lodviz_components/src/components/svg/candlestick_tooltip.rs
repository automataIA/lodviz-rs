/// Tooltip overlay for candlestick charts with bisect hit-testing on timestamp
use leptos::prelude::*;
use lodviz_core::core::data::OhlcBar;
use lodviz_core::core::scale::{LinearScale, Scale};
use lodviz_core::core::theme::Margin;

/// Tooltip overlay for candlestick / OHLC charts
#[component]
pub fn CandlestickTooltip(
    /// Visible OHLC bars (sorted by timestamp, already downsampled if needed)
    bars: Memo<Vec<OhlcBar>>,
    /// X scale mapping timestamp → pixel x
    x_scale: Memo<LinearScale>,
    /// Y scale mapping price → pixel y (unused for hit-test but passed for completeness)
    #[allow(unused)]
    y_scale: Memo<LinearScale>,
    /// Pixel width of each candlestick body (for highlight rect)
    bar_pixel_width: Memo<f64>,
    /// Inner width of the chart area
    inner_width: Memo<f64>,
    /// Inner height of the chart area
    inner_height: Memo<f64>,
    /// Chart margins (to correct SVG offset coordinates)
    margin: Memo<Margin>,
) -> impl IntoView {
    let (mouse_pos, set_mouse_pos) = signal(None::<(f64, f64)>);

    // Bisect: find the bar whose timestamp is closest to the mouse x
    let hovered_idx = Memo::new(move |_| {
        let (mx, _) = mouse_pos.get()?;
        let xs = x_scale.get();
        let data_x = xs.inverse(mx);
        let all_bars = bars.get();
        if all_bars.is_empty() {
            return None;
        }
        let idx = all_bars.partition_point(|b| b.timestamp < data_x);
        let best = if idx == 0 {
            0
        } else if idx >= all_bars.len() {
            all_bars.len() - 1
        } else {
            let dl = (all_bars[idx - 1].timestamp - data_x).abs();
            let dr = (all_bars[idx].timestamp - data_x).abs();
            if dl <= dr {
                idx - 1
            } else {
                idx
            }
        };
        Some(best)
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

        // Tooltip rendering (pointer-events disabled)
        {move || {
            let idx = hovered_idx.get()?;
            let (mx, my) = mouse_pos.get()?;
            let all_bars = bars.get();
            let bar = all_bars.get(idx)?;
            let xs = x_scale.get();
            let w = inner_width.get();
            let h = inner_height.get();
            let bpw = bar_pixel_width.get();
            let bar_cx = xs.map(bar.timestamp);
            let hl_x = bar_cx - bpw / 2.0;
            let is_bullish = bar.is_bullish();
            let badge_color = if is_bullish { "#26a69a" } else { "#ef5350" };
            let badge_text = if is_bullish { "▲ Bullish" } else { "▼ Bearish" };
            let delta = bar.close - bar.open;
            let pct = if bar.open.abs() > f64::EPSILON { delta / bar.open * 100.0 } else { 0.0 };
            let box_w = 175.0_f64;
            let box_h = 90.0_f64;
            let padding = 8.0_f64;
            let row_h = 16.0_f64;
            let box_x = if mx + box_w + 10.0 > w { mx - box_w - 10.0 } else { mx + 10.0 };
            let box_y = if my + box_h + 10.0 > h { my - box_h - 10.0 } else { my + 10.0 };
            Some(

                // Vertical highlight centered on the candlestick

                // Badge color and text

                // Delta and percentage

                // Tooltip box sizing

                // Auto-flip near edges

                view! {
                    <g class="candlestick-tooltip-overlay" style="pointer-events: none;">
                        // Vertical highlight band
                        <rect
                            x=format!("{hl_x:.2}")
                            y="0"
                            width=format!("{bpw:.2}")
                            height=format!("{h:.2}")
                            fill="white"
                            opacity="0.2"
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

                        // Left color stripe
                        <rect
                            x=format!("{box_x:.2}")
                            y=format!("{box_y:.2}")
                            width="3"
                            height=box_h
                            rx="4"
                            fill=badge_color
                        />

                        // Header row: bar index + badge
                        <text
                            x=format!("{:.2}", box_x + padding)
                            y=format!("{:.2}", box_y + padding + 12.0)
                            font-size="11"
                            fill="white"
                            font-family="monospace"
                            font-weight="bold"
                        >
                            {format!("Bar #{idx}")}
                        </text>
                        <text
                            x=format!("{:.2}", box_x + box_w - padding)
                            y=format!("{:.2}", box_y + padding + 12.0)
                            font-size="10"
                            fill=badge_color
                            font-family="monospace"
                            text-anchor="end"
                        >
                            {badge_text}
                        </text>

                        // O / C row
                        <text
                            x=format!("{:.2}", box_x + padding)
                            y=format!("{:.2}", box_y + padding + 12.0 + row_h)
                            font-size="10"
                            fill="#ddd"
                            font-family="monospace"
                        >
                            {format!("O: {:.2}   C: {:.2}", bar.open, bar.close)}
                        </text>

                        // H / L row
                        <text
                            x=format!("{:.2}", box_x + padding)
                            y=format!("{:.2}", box_y + padding + 12.0 + row_h * 2.0)
                            font-size="10"
                            fill="#ddd"
                            font-family="monospace"
                        >
                            {format!("H: {:.2}   L: {:.2}", bar.high, bar.low)}
                        </text>

                        // Δ row
                        <text
                            x=format!("{:.2}", box_x + padding)
                            y=format!("{:.2}", box_y + padding + 12.0 + row_h * 3.0)
                            font-size="10"
                            fill=badge_color
                            font-family="monospace"
                        >
                            {format!("Δ: {:+.2} ({:+.1}%)", delta, pct)}
                        </text>
                    </g>
                },
            )
        }}
    }
}

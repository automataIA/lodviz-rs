/// SVG overlay components: trend lines, SMA, etc.
///
/// These components render pure SVG elements and are designed to be placed
/// inside an existing chart's `<g>` group that already has scales applied.
use leptos::prelude::*;
use lodviz_core::algorithms::statistics::{linear_regression, sma};
use lodviz_core::core::scale::{LinearScale, Scale};

/// Trend line (OLS linear regression) overlaid on a chart
///
/// Renders a dashed `<line>` spanning the full X domain.
#[component]
pub fn TrendLine(
    /// Raw (x, y) data points for regression
    points: Signal<Vec<(f64, f64)>>,
    /// X scale (Memo from parent chart)
    x_scale: Memo<LinearScale>,
    /// Y scale (Memo from parent chart)
    y_scale: Memo<LinearScale>,
    /// Line color
    #[prop(default = "#ff6b6b")]
    color: &'static str,
    /// Stroke width (px)
    #[prop(default = 2.0)]
    stroke_width: f64,
    /// SVG stroke-dasharray pattern
    #[prop(default = "6,4")]
    dash: &'static str,
) -> impl IntoView {
    let line_coords = Memo::new(move |_| {
        let pts = points.get();
        let (b0, b1) = linear_regression(&pts)?;
        let xs = x_scale.get();
        let ys = y_scale.get();
        let (x_domain_min, x_domain_max) = xs.domain();
        let y0 = b0 + b1 * x_domain_min;
        let y1 = b0 + b1 * x_domain_max;
        Some((
            xs.map(x_domain_min),
            ys.map(y0),
            xs.map(x_domain_max),
            ys.map(y1),
        ))
    });

    view! {
        {move || {
            line_coords
                .get()
                .map(|(x1, y1, x2, y2)| {
                    view! {
                        <line
                            x1=format!("{x1:.2}")
                            y1=format!("{y1:.2}")
                            x2=format!("{x2:.2}")
                            y2=format!("{y2:.2}")
                            stroke=color
                            stroke-width=stroke_width
                            stroke-dasharray=dash
                            stroke-linecap="round"
                        />
                    }
                })
        }}
    }
}

/// Simple Moving Average overlay
///
/// Computes SMA on `data` with the given `window` and renders a polyline
/// at x positions given by `xs` (must be same length as `data`).
#[component]
pub fn SmaOverlay(
    /// Y values (same index as `xs`)
    data: Signal<Vec<f64>>,
    /// X values corresponding to each data point
    xs: Signal<Vec<f64>>,
    /// SMA window size
    #[prop(default = 7)]
    window: usize,
    /// X scale from parent chart
    x_scale: Memo<LinearScale>,
    /// Y scale from parent chart
    y_scale: Memo<LinearScale>,
    /// Line color
    #[prop(default = "#4ecdc4")]
    color: &'static str,
    /// Stroke width (px)
    #[prop(default = 2.0)]
    stroke_width: f64,
) -> impl IntoView {
    let path_d = Memo::new(move |_| {
        let y_vals = data.get();
        let x_vals = xs.get();
        let smoothed = sma(&y_vals, window);
        if smoothed.is_empty() {
            return String::new();
        }
        // SMA produces len(data) - window + 1 values, aligned to indices [window-1..]
        let offset = window.saturating_sub(1);
        let x_scale_val = x_scale.get();
        let y_scale_val = y_scale.get();

        smoothed
            .iter()
            .enumerate()
            .filter_map(|(i, &y)| {
                let xi = x_vals.get(i + offset).copied()?;
                Some((x_scale_val.map(xi), y_scale_val.map(y)))
            })
            .enumerate()
            .map(|(k, (px, py))| {
                if k == 0 {
                    format!("M {px:.2} {py:.2}")
                } else {
                    format!(" L {px:.2} {py:.2}")
                }
            })
            .collect::<String>()
    });

    view! {
        {move || {
            let d = path_d.get();
            if d.is_empty() {
                ().into_any()
            } else {
                view! {
                    <path
                        d=d
                        fill="none"
                        stroke=color
                        stroke-width=stroke_width
                        stroke-linejoin="round"
                        stroke-linecap="round"
                    />
                }
                    .into_any()
            }
        }}
    }
}

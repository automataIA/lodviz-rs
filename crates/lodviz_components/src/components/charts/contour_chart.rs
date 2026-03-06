/// Contour chart component (marching squares iso-lines)
use crate::components::svg::colorbar::ColorBar;
use crate::hooks::use_container_size;
use leptos::prelude::*;
use lodviz_core::algorithms::contour::{
    close_open_path_at_boundary, contour_to_svg_path, marching_squares,
};
use lodviz_core::core::color_map::{ColorMap, SequentialColorMap};
use lodviz_core::core::data::GridData;
use lodviz_core::core::theme::ChartConfig;

/// ContourChart: iso-line (or filled iso-band) visualization of a 2-D scalar field.
///
/// Features:
/// - Marching squares contour extraction
/// - Optional filled iso-bands between consecutive levels
/// - Optional ColorBar on the right
/// - Configurable number of levels
/// - Responsive via `use_container_size()`
#[component]
pub fn ContourChart(
    /// 2-D scalar grid data
    data: Signal<GridData>,
    /// Number of iso-levels to extract
    #[prop(default = 10)]
    levels: usize,
    /// If true, render filled iso-bands between consecutive levels
    #[prop(default = true)]
    filled: bool,
    /// Color map for encoding levels as colors
    #[prop(default = ColorMap::Sequential(SequentialColorMap::Viridis))]
    color_map: ColorMap,
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

    let colorbar_w = if show_colorbar { 60.0 } else { 0.0 };
    let margin_top = 30.0_f64;
    let margin_bottom = 30.0_f64;
    let margin_left = 20.0_f64;
    let margin_right = colorbar_w + 10.0;

    let inner_width =
        Memo::new(move |_| (chart_width.get() as f64 - margin_left - margin_right).max(10.0));
    let inner_height =
        Memo::new(move |_| (chart_height.get() as f64 - margin_top - margin_bottom).max(10.0));

    let grid_stats = Memo::new(move |_| {
        let g = data.get();
        let min = g
            .values
            .iter()
            .flatten()
            .cloned()
            .fold(f64::INFINITY, f64::min);
        let max_v = g
            .values
            .iter()
            .flatten()
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max);
        let min = if min.is_infinite() { 0.0 } else { min };
        let max_v = if max_v.is_infinite() { 1.0 } else { max_v };
        (min, max_v)
    });

    let color_map_lines = color_map.clone();
    let color_map_bar = color_map.clone();

    // Pre-compute contour data
    let contour_data = Memo::new(move |_| {
        let g = data.get();
        let (min, max_v) = grid_stats.get();
        let iw = inner_width.get();
        let ih = inner_height.get();
        if g.values.is_empty() || levels == 0 {
            return vec![];
        }
        let level_vals: Vec<f64> = (0..levels)
            .map(|i| min + (max_v - min) * i as f64 / (levels - 1).max(1) as f64)
            .collect();
        marching_squares(&g.values, &level_vals, (0.0, iw), (0.0, ih))
    });

    let clip_id = format!("contour-clip-{}", uuid::Uuid::new_v4().simple());
    let a11y_title_id = format!("chart-title-{}", uuid::Uuid::new_v4().as_simple());
    let a11y_desc_id = format!("chart-desc-{}", uuid::Uuid::new_v4().as_simple());
    let a11y_labelledby = format!("{} {}", a11y_title_id, a11y_desc_id);

    view! {
        <div
            class="contour-chart"
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
                    <title id=a11y_title_id>
                        {move || final_title.get().unwrap_or("Contour chart".to_string())}
                    </title>
                    <desc id=a11y_desc_id>"Contour chart showing iso-level curves of a two-dimensional scalar field."</desc>

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

                        // Contour lines / filled bands
                        {move || {
                            let contours = contour_data.get();
                            let (min, max_v) = grid_stats.get();
                            let range = (max_v - min).max(1e-12);
                            let n = contours.len();
                            let cm = color_map_lines.clone();
                            let iw = inner_width.get();
                            let ih = inner_height.get();
                            if filled && n >= 2 {
                                contours
                                    .windows(2)
                                    .enumerate()
                                    .map(|(i, pair)| {
                                        let t = if n > 1 { i as f64 / (n - 2) as f64 } else { 0.5 };
                                        let color = cm.map(t);
                                        let upper = &pair[1];
                                        let paths = upper
                                            .paths
                                            .iter()
                                            .map(|p| {
                                                let closed = close_open_path_at_boundary(
                                                    p,
                                                    (0.0, iw),
                                                    (0.0, ih),
                                                );
                                                let svg = contour_to_svg_path(&closed);
                                                if svg.is_empty() { svg } else { format!("{svg} Z") }
                                            })
                                            .filter(|s| !s.is_empty())
                                            .collect::<Vec<_>>()
                                            .join(" ");
                                        if paths.is_empty() {
                                            view! { <g></g> }.into_any()
                                        } else {
                                            view! {
                                                <path
                                                    d=paths
                                                    fill=color
                                                    opacity="0.85"
                                                    clip-path=format!("url(#{})", clip_id)
                                                    pointer-events="none"
                                                />
                                            }
                                                .into_any()
                                        }
                                    })
                                    .collect_view()
                            } else {
                                contours
                                    .iter()
                                    .enumerate()
                                    .map(|(i, cl)| {
                                        let t = if n > 1 { i as f64 / (n - 1) as f64 } else { 0.5 };
                                        let t_clamped = ((cl.level - min) / range).clamp(0.0, 1.0);
                                        let color = cm.map(t_clamped);
                                        let _ = t;
                                        let paths = cl
                                            .paths
                                            .iter()
                                            .map(|p| contour_to_svg_path(p))
                                            .filter(|s| !s.is_empty())
                                            .collect::<Vec<_>>()
                                            .join(" ");
                                        if paths.is_empty() {
                                            view! { <g></g> }.into_any()
                                        } else {
                                            view! {
                                                <path
                                                    d=paths
                                                    fill="none"
                                                    stroke=color
                                                    stroke-width="1.5"
                                                    clip-path=format!("url(#{})", clip_id)
                                                    pointer-events="none"
                                                />
                                            }
                                                .into_any()
                                        }
                                    })
                                    .collect_view()
                            }
                        }}

                        // ColorBar
                        {move || {
                            if !show_colorbar {
                                return None;
                            }
                            let (min, max_v) = grid_stats.get();
                            let ih = inner_height.get();
                            let iw = inner_width.get();
                            let th = theme.get();
                            Some(
                                view! {
                                    <g transform=format!("translate({:.2}, 0)", iw + 8.0)>
                                        <ColorBar
                                            color_map=color_map_bar.clone()
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
                    </g>
                </svg>
            </div>
        </div>
    }
}

/// ScatterChart component with LTTB downsampling and multi-series support
use crate::components::interaction::zoom_pan::{ZoomPan, ZoomTransform};
use crate::components::svg::axis::{Axis, AxisOrientation};
use crate::components::svg::grid::Grid;
use crate::components::svg::legend::{estimate_legend_width, Legend, LegendItem, LegendPosition};
use crate::components::svg::tooltip::{Tooltip, TooltipMode};
use crate::hooks::use_container_size;
use leptos::prelude::*;
use lodviz_core::algorithms::lttb::lttb_downsample;
use lodviz_core::core::a11y;
use lodviz_core::core::data::Dataset;
use lodviz_core::core::mark::Mark;
use lodviz_core::core::scale::{LinearScale, Scale};
use lodviz_core::core::theme::{ChartConfig, GridStyle};

/// ScatterChart component for rendering scatter plots with automatic downsampling
///
/// Features:
/// - Multi-series support via `Dataset`
/// - Automatic LTTB downsampling for series > 1000 points
/// - Interactive legend with click-to-toggle
/// - Optional axis labels
/// - Point opacity from theme (configurable via ChartTheme.point_opacity)
#[component]
pub fn ScatterChart(
    /// Dataset containing one or more series
    data: Signal<Dataset>,
    /// Width of the chart (optional, uses card dimensions if in a DraggableCard)
    #[prop(optional)]
    width: Option<u32>,
    /// Height of the chart (optional, uses card dimensions if in a DraggableCard)
    #[prop(optional)]
    height: Option<u32>,
    /// Chart title (optional)
    #[prop(optional)]
    title: Option<String>,
    /// Show grid background
    #[prop(default = true)]
    show_grid: bool,
    /// X axis label (optional)
    #[prop(optional, into)]
    x_label: Option<String>,
    /// Y axis label (optional)
    #[prop(optional, into)]
    y_label: Option<String>,
    /// Chart configuration (overrides specific props if present)
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

    // Processed data: LTTB downsample per series (defined early — needed by legend_items before margin)
    let processed_data = Memo::new(move |_| {
        let dataset = data.get();
        dataset
            .series
            .iter()
            .map(|s| {
                let points = if s.data.len() > 1000 {
                    lttb_downsample(&s.data, 1000)
                } else {
                    s.data.clone()
                };
                (s.name.clone(), points)
            })
            .collect::<Vec<_>>()
    });

    // Legend items — defined early so margin can adapt when legend_outside is enabled
    let legend_items = Signal::derive(move || {
        let series = processed_data.get();
        let vis = series_visibility.get();
        let th = theme.get();
        series
            .iter()
            .enumerate()
            .map(|(i, (name, _))| LegendItem {
                name: name.clone(),
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

    // Initial domain calculation (full extent)
    let initial_transform = Memo::new(move |_| {
        let series = processed_data.get();
        // Default 0..1 if empty
        let mut x_min = f64::INFINITY;
        let mut x_max = f64::NEG_INFINITY;
        let mut y_min = f64::INFINITY;
        let mut y_max = f64::NEG_INFINITY;

        let all_points = series.iter().flat_map(|(_, pts)| pts.iter());
        for p in all_points {
            if p.x < x_min {
                x_min = p.x;
            }
            if p.x > x_max {
                x_max = p.x;
            }
            if p.y < y_min {
                y_min = p.y;
            }
            if p.y > y_max {
                y_max = p.y;
            }
        }

        if x_min >= x_max {
            x_min = 0.0;
            x_max = 1.0;
        }
        if y_min >= y_max {
            y_min = 0.0;
            y_max = 1.0;
        }

        ZoomTransform::from_domain(x_min, x_max, y_min, y_max)
    });

    // Zoom state
    let zoom_transform = RwSignal::new(ZoomTransform::from_domain(0.0, 1.0, 0.0, 1.0));

    // Reset zoom when data changes drastically
    Effect::new(move |_| {
        let new_initial = initial_transform.get();
        zoom_transform.set(new_initial);
    });

    // Scales computed from ZoomTransform
    let x_scale = Memo::new(move |_| {
        let t = zoom_transform.get();
        let w = inner_width.get();
        LinearScale::new((t.x_min, t.x_max), (0.0, w))
    });

    let y_scale = Memo::new(move |_| {
        let t = zoom_transform.get();
        let h = inner_height.get();
        LinearScale::new((t.y_min, t.y_max), (h, 0.0))
    });

    let x_tick_count = Memo::new(move |_| (inner_width.get() / 100.0).max(2.0) as usize);
    let y_tick_count = Memo::new(move |_| (inner_height.get() / 50.0).max(2.0) as usize);

    // Accessibility
    let chart_description = Memo::new(move |_| {
        let series = processed_data.get();
        let total_points: usize = series.iter().map(|(_, pts)| pts.len()).sum();
        let mut desc = a11y::generate_chart_description(Mark::Point, total_points, None, None);
        if series.len() > 1 {
            desc.push_str(&format!(" {} series: ", series.len()));
            let names: Vec<_> = series.iter().map(|(n, _)| n.as_str()).collect();
            desc.push_str(&names.join(", "));
            desc.push('.');
        }
        desc
    });

    let aria_label = Memo::new(move |_| {
        final_title
            .get()
            .unwrap_or_else(|| "Scatter chart".to_string())
    });

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

    let (focused_index, set_focused_index) = signal(None::<usize>);

    // Tooltip data
    let tooltip_series = Memo::new(move |_| {
        let series = processed_data.get();
        let vis = series_visibility.get();
        series
            .iter()
            .enumerate()
            .filter(|(i, _)| vis.get(*i).copied().unwrap_or(true))
            .map(|(_, (name, pts))| (name.clone(), pts.clone()))
            .collect::<Vec<_>>()
    });

    let tooltip_colors = Memo::new(move |_| {
        let series = processed_data.get();
        let vis = series_visibility.get();
        let th = theme.get();
        series
            .iter()
            .enumerate()
            .filter(|(i, _)| vis.get(*i).copied().unwrap_or(true))
            .map(|(i, _)| th.palette[i % th.palette.len()].clone())
            .collect::<Vec<_>>()
    });

    let x_label_clone = x_label.clone();
    let y_label_clone = y_label.clone();

    // Unique ID for clip path
    let clip_id = format!("clip-{}", uuid::Uuid::new_v4());

    // Cursor tracking for tooltips
    let (cursor_norm, set_cursor_norm) = signal(None::<(f64, f64)>);

    // Derived cursor X for tooltips (normalized X)
    let cursor_x = Memo::new(move |_| cursor_norm.get().map(|(x, _)| x));
    // Derived cursor Y for tooltips (normalized Y)
    let cursor_y = Memo::new(move |_| cursor_norm.get().map(|(_, y)| y));

    view! {
        <div
            class="scatter-chart"
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
                    style:outline=move || { focused_index.get().map(|_| "2px solid #4992ff") }
                    on:keydown=move |ev| {
                        let series = processed_data.get();
                        let vis = series_visibility.get();
                        let first_visible = series
                            .iter()
                            .enumerate()
                            .find(|(i, _)| vis.get(*i).copied().unwrap_or(true))
                            .map(|(_, (_, pts))| pts.clone());
                        let Some(data_points) = first_visible else { return };
                        if data_points.is_empty() {
                            return;
                        }
                        let key = ev.key();
                        match key.as_str() {
                            "ArrowRight" => {
                                ev.prevent_default();
                                let next = match focused_index.get() {
                                    Some(i) => (i + 1).min(data_points.len() - 1),
                                    None => 0,
                                };
                                set_focused_index.set(Some(next));
                            }
                            "ArrowLeft" => {
                                ev.prevent_default();
                                let prev = match focused_index.get() {
                                    Some(i) => i.saturating_sub(1),
                                    None => 0,
                                };
                                set_focused_index.set(Some(prev));
                            }
                            "Escape" => {
                                set_focused_index.set(None);
                            }
                            _ => {}
                        }
                    }
                >
                    <title>{move || aria_label.get()}</title>
                    <desc>{move || chart_description.get()}</desc>
                    <g transform=move || {
                        format!("translate({}, {})", margin.get().left, margin.get().top)
                    }>
                        <defs>
                            <clipPath id=clip_id.clone()>
                                <rect
                                    x="0"
                                    y="0"
                                    width=move || inner_width.get()
                                    height=move || inner_height.get()
                                ></rect>
                            </clipPath>
                        </defs>

                        // Grid (optional)
                        {move || {
                            let gs = grid_style.get();
                            (gs.show_x || gs.show_y)
                                .then(|| {
                                    view! {
                                        <Grid
                                            x_scale=x_scale.get()
                                            y_scale=y_scale.get()
                                            tick_count=x_tick_count.get()
                                            width=inner_width.get()
                                            height=inner_height.get()
                                            style=gs
                                        />
                                    }
                                })
                        }}
                        // Scatter points (one group per series)
                        {move || {
                            let series = processed_data.get();
                            let vis = series_visibility.get();
                            let xs = x_scale.get();
                            let ys = y_scale.get();
                            let th = theme.get();
                            series
                                .iter()
                                .enumerate()
                                .map(|(i, (_, points))| {
                                    let visible = vis.get(i).copied().unwrap_or(true);
                                    let color = th.palette[i % th.palette.len()].clone();
                                    let display = if visible { "inline" } else { "none" };
                                    let circles = points
                                        .iter()
                                        .map(|point| {
                                            let cx = xs.map(point.x);
                                            let cy = ys.map(point.y);
                                            view! {
                                                <circle
                                                    cx=format!("{cx:.2}")
                                                    cy=format!("{cy:.2}")
                                                    r=th.point_radius
                                                    fill=color.clone()
                                                    opacity=th.point_opacity
                                                />
                                            }
                                        })
                                        .collect_view();
                                    view! {
                                        <g
                                            class="scatter-series"
                                            display=display
                                            clip-path=format!("url(#{})", clip_id)
                                        >
                                            {circles}
                                        </g>
                                    }
                                })
                                .collect_view()
                        }}
                        // Keyboard focus indicator
                        {move || {
                            let series = processed_data.get();
                            let vis = series_visibility.get();
                            let th = theme.get();
                            focused_index
                                .get()
                                .and_then(|idx| {
                                    let (si, (_, points)) = series
                                        .iter()
                                        .enumerate()
                                        .find(|(i, _)| vis.get(*i).copied().unwrap_or(true))?;
                                    let point = points.get(idx)?;
                                    let cx = x_scale.get().map(point.x);
                                    let cy = y_scale.get().map(point.y);
                                    let desc = a11y::describe_point(point, idx, points.len());
                                    let color = th.palette[si % th.palette.len()].clone();
                                    Some(
                                        view! {
                                            <g>
                                                <circle
                                                    cx=format!("{cx:.2}")
                                                    cy=format!("{cy:.2}")
                                                    r=8
                                                    fill="none"
                                                    stroke=color
                                                    stroke-width=2
                                                />
                                                <text
                                                    x=format!("{cx:.2}")
                                                    y=format!("{:.2}", cy - 14.0)
                                                    text-anchor="middle"
                                                    font-size="11"
                                                    fill=th.text_color.clone()
                                                    role="status"
                                                    aria-live="polite"
                                                >
                                                    {desc}
                                                </text>
                                            </g>
                                        },
                                    )
                                })
                        }}
                        // X axis (bottom)
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
                        // Y axis (left)
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

                        // Tooltip overlay (must be last to receive mouse events)
                        <Tooltip
                            series_data=tooltip_series
                            series_colors=tooltip_colors
                            x_scale=x_scale
                            y_scale=y_scale
                            inner_width=inner_width
                            inner_height=inner_height
                            cursor_normalized_x=cursor_x
                            cursor_normalized_y=cursor_y
                            mode=TooltipMode::Euclidean
                        />

                        // Zoom/Pan overlay
                        <ZoomPan
                            transform=zoom_transform
                            original=initial_transform
                            inner_width=inner_width
                            inner_height=inner_height
                            set_cursor=set_cursor_norm
                        />

                        // SVG Legend overlay (must be last to render on top)
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

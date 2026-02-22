/// PieChart / DonutChart component
use crate::hooks::use_container_size;
use leptos::prelude::*;
use lodviz_core::algorithms::arc::{arc_centroid, arc_path, compute_arcs};
use lodviz_core::core::theme::ChartConfig;

/// A single pie data entry: label + value
#[derive(Debug, Clone)]
pub struct PieEntry {
    /// Category label
    pub label: String,
    /// Numerical value (determines angular size)
    pub value: f64,
}

/// PieChart component for rendering pie and donut charts
///
/// Features:
/// - Pie or donut mode (controlled by `donut` prop)
/// - Percentage labels on each slice
/// - Hover tooltip with value details
/// - A11y: ARIA labels per slice, keyboard navigable
/// - Responsive SVG rendering
#[component]
pub fn PieChart(
    /// Data entries (label, value pairs)
    data: Signal<Vec<PieEntry>>,
    /// Width (optional)
    #[prop(optional)]
    width: Option<u32>,
    /// Height (optional)
    #[prop(optional)]
    height: Option<u32>,
    /// Chart title
    #[prop(optional)]
    title: Option<String>,
    /// Donut mode (true = hollow center)
    #[prop(default = false)]
    donut: bool,
    /// Inner radius ratio for donut (0.0 to 1.0, fraction of outer radius)
    #[prop(default = 0.5)]
    inner_ratio: f64,
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
        config.get().width.or(width).unwrap_or(400)
    });

    let chart_height = Memo::new(move |_| {
        let measured = container_height.get();
        if measured > 0.0 {
            return measured as u32;
        }
        config.get().height.or(height).unwrap_or(400)
    });

    let final_title = Memo::new(move |_| config.get().title.or(title.clone()));

    // Compute center and radius
    let cx = Memo::new(move |_| chart_width.get() as f64 / 2.0);
    let cy = Memo::new(move |_| chart_height.get() as f64 / 2.0);
    let outer_radius = Memo::new(move |_| {
        let w = chart_width.get() as f64;
        let h = chart_height.get() as f64;
        (w.min(h) / 2.0 - 40.0).max(20.0) // 40px margin for labels
    });

    // Hover state
    let (hovered_index, set_hovered_index) = signal(None::<usize>);

    let aria_label = Memo::new(move |_| {
        final_title.get().unwrap_or_else(|| {
            if donut {
                "Donut chart".to_string()
            } else {
                "Pie chart".to_string()
            }
        })
    });

    view! {
        <div
            class="pie-chart"
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
                    style="width: 100%; height: 100%; display: block; outline: none;"
                >
                    <title>{move || aria_label.get()}</title>
                    // Slices
                    {move || {
                        let entries = data.get();
                        let values: Vec<f64> = entries.iter().map(|e| e.value).collect();
                        let arcs = compute_arcs(&values);
                        let th = theme.get();
                        let center_x = cx.get();
                        let center_y = cy.get();
                        let r = outer_radius.get();
                        let ir = if donut { r * inner_ratio } else { 0.0 };
                        let label_r = if donut { (r + ir) / 2.0 } else { r * 0.65 };
                        let hover = hovered_index.get();
                        let mut arc_idx = 0;
                        let mut views = Vec::new();
                        for (entry_idx, entry) in entries.iter().enumerate() {
                            if entry.value <= 0.0 {
                                continue;
                            }
                            let arc = &arcs[arc_idx];
                            let color = th.palette[entry_idx % th.palette.len()].clone();
                            let is_hovered = hover == Some(entry_idx);
                            let scale_transform = if is_hovered {
                                let mid = arc.mid_angle();
                                let offset = 6.0;
                                let dx = offset * mid.cos();
                                let dy = offset * mid.sin();
                                format!("translate({dx:.2}, {dy:.2})")
                            } else {
                                String::new()
                            };
                            let path_d = arc_path(
                                center_x,
                                center_y,
                                r,
                                ir,
                                arc.start_angle,
                                arc.end_angle,
                            );
                            let label_text = format!("{:.0}%", arc.percentage);
                            let (lx, ly) = arc_centroid(
                                center_x,
                                center_y,
                                label_r,
                                arc.start_angle,
                                arc.end_angle,
                            );
                            let aria = format!(
                                "{}: {:.1} ({:.1}%)",
                                entry.label,
                                entry.value,
                                arc.percentage,
                            );
                            let idx = entry_idx;
                            views
                                .push(

                                    // Build slices - we need to track the mapping from arcs back to entries
                                    // since compute_arcs skips non-positive values

                                    // Hover scale effect

                                    view! {
                                        <g
                                            transform=scale_transform
                                            style="cursor: pointer;"
                                            on:mouseenter=move |_| set_hovered_index.set(Some(idx))
                                            on:mouseleave=move |_| set_hovered_index.set(None)
                                        >
                                            <path
                                                d=path_d
                                                fill=color
                                                stroke=th.background_color.clone()
                                                stroke-width="2"
                                                aria-label=aria
                                            />
                                            // Percentage label (only show if slice is big enough)
                                            {if arc.percentage >= 5.0 {
                                                Some(
                                                    view! {
                                                        <text
                                                            x=format!("{lx:.2}")
                                                            y=format!("{ly:.2}")
                                                            text-anchor="middle"
                                                            dominant-baseline="central"
                                                            font-size=format!("{}", th.axis_font_size)
                                                            fill=th.text_color.clone()
                                                            font-weight="bold"
                                                            pointer-events="none"
                                                        >
                                                            {label_text}
                                                        </text>
                                                    },
                                                )
                                            } else {
                                                None
                                            }}
                                        </g>
                                    },
                                );
                            arc_idx += 1;
                        }
                        views.collect_view()
                    }}
                    // Tooltip on hover
                    {move || {
                        let entries = data.get();
                        let th = theme.get();
                        hovered_index
                            .get()
                            .and_then(|idx| {
                                let entry = entries.get(idx)?;
                                let values: Vec<f64> = entries.iter().map(|e| e.value).collect();
                                let total: f64 = values.iter().filter(|v| **v > 0.0).sum();
                                let pct = if total > 0.0 {
                                    (entry.value / total) * 100.0
                                } else {
                                    0.0
                                };
                                Some(

                                    view! {
                                        <g>
                                            <rect
                                                x=format!("{:.2}", cx.get() - 60.0)
                                                y=format!("{:.2}", cy.get() - 20.0)
                                                width="120"
                                                height="40"
                                                fill=th.background_color.clone()
                                                stroke=th.axis_color.clone()
                                                rx="4"
                                                opacity="0.9"
                                            />
                                            <text
                                                x=format!("{:.2}", cx.get())
                                                y=format!("{:.2}", cy.get() - 4.0)
                                                text-anchor="middle"
                                                font-size=format!("{}", th.axis_font_size)
                                                fill=th.text_color.clone()
                                                font-weight="bold"
                                            >
                                                {entry.label.clone()}
                                            </text>
                                            <text
                                                x=format!("{:.2}", cx.get())
                                                y=format!("{:.2}", cy.get() + 12.0)
                                                text-anchor="middle"
                                                font-size=format!("{}", th.axis_font_size)
                                                fill=th.text_color.clone()
                                            >
                                                {format!("{:.1} ({:.1}%)", entry.value, pct)}
                                            </text>
                                        </g>
                                    },
                                )
                            })
                    }}
                    // Legend below the chart
                    {move || {
                        let entries = data.get();
                        let th = theme.get();
                        let start_y = chart_height.get() as f64 - 20.0;
                        let total_width = entries.len() as f64 * 100.0;
                        let start_x = (chart_width.get() as f64 - total_width) / 2.0;
                        entries
                            .iter()
                            .enumerate()
                            .map(|(i, entry)| {
                                let color = th.palette[i % th.palette.len()].clone();
                                let x = start_x + i as f64 * 100.0;

                                view! {
                                    <g>
                                        <rect
                                            x=format!("{x:.2}")
                                            y=format!("{:.2}", start_y)
                                            width="10"
                                            height="10"
                                            fill=color
                                            rx="2"
                                        />
                                        <text
                                            x=format!("{:.2}", x + 14.0)
                                            y=format!("{:.2}", start_y + 9.0)
                                            font-size=format!("{}", th.axis_font_size)
                                            fill=th.text_color.clone()
                                        >
                                            {entry.label.clone()}
                                        </text>
                                    </g>
                                }
                            })
                            .collect_view()
                    }}
                </svg>
            </div>
        </div>
    }
}

/// Chord diagram component
use crate::hooks::use_container_size;
use leptos::prelude::*;
use lodviz_core::algorithms::chord_layout::layout_chord;
use lodviz_core::core::data::ChordData;
use lodviz_core::core::theme::ChartConfig;

/// ChordChart: circular flow diagram showing relationships between groups.
///
/// Features:
/// - Arc bands proportional to row totals
/// - Quadratic Bézier ribbons for flows
/// - Group labels outside the arcs
/// - Hover tooltip for arcs and ribbons
/// - Responsive via `use_container_size()`
#[component]
pub fn ChordChart(
    /// Chord data: square flow matrix + labels + optional colors
    data: Signal<ChordData>,
    /// Gap in degrees between adjacent arc segments
    #[prop(default = 2.0)]
    gap_degrees: f64,
    /// Opacity of ribbon paths
    #[prop(default = 0.65)]
    ribbon_opacity: f64,
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
        config.get().width.or(width).unwrap_or(500)
    });
    let chart_height = Memo::new(move |_| {
        let m = container_height.get();
        if m > 0.0 {
            return m as u32;
        }
        config.get().height.or(height).unwrap_or(500)
    });

    let final_title = Memo::new(move |_| config.get().title);

    // Layout computed from current dimensions and data
    let layout = Memo::new(move |_| {
        let d = data.get();
        let cw = chart_width.get() as f64;
        let ch = chart_height.get() as f64;
        // Title h reduces space; leave 30px margin around
        let title_h = if final_title.get().is_some() {
            40.0
        } else {
            0.0
        };
        let available = (cw.min(ch - title_h) - 60.0).max(40.0);
        let radius = available / 2.0;
        let inner_radius = radius * 0.75;
        let colors: Option<Vec<String>> = d.colors.clone();
        layout_chord(
            &d.matrix,
            &d.labels,
            colors.as_deref(),
            gap_degrees,
            radius,
            inner_radius,
        )
    });

    // Centre of the chord diagram in SVG coordinates
    let center = Memo::new(move |_| {
        let cw = chart_width.get() as f64;
        let ch = chart_height.get() as f64;
        let title_h = if final_title.get().is_some() {
            40.0
        } else {
            0.0
        };
        let available = (cw.min(ch - title_h) - 60.0).max(40.0);
        let radius = available / 2.0;
        (cw / 2.0 - radius, title_h + 30.0)
    });

    // Hover state
    let (hover_label, set_hover_label) = signal(None::<String>);
    let (hover_value, set_hover_value) = signal(None::<f64>);
    let (hover_x, set_hover_x) = signal(0.0_f64);
    let (hover_y, set_hover_y) = signal(0.0_f64);

    let a11y_title_id = format!("chart-title-{}", uuid::Uuid::new_v4().as_simple());
    let a11y_desc_id = format!("chart-desc-{}", uuid::Uuid::new_v4().as_simple());
    let a11y_labelledby = format!("{} {}", a11y_title_id, a11y_desc_id);

    view! {
        <div
            class="chord-chart"
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
                        {move || final_title.get().unwrap_or("Chord diagram".to_string())}
                    </title>
                    <desc id=a11y_desc_id>"Chord diagram showing the magnitude of flows between groups in a circular layout."</desc>

                    {move || {
                        let lay = layout.get();
                        let (cx_off, cy_off) = center.get();
                        let th = theme.get();
                        let outer_r = lay
                            .arcs
                            .first()
                            .map(|_a| {
                                let cw = chart_width.get() as f64;
                                let ch = chart_height.get() as f64;
                                let title_h = if final_title.get().is_some() { 40.0 } else { 0.0 };
                                let available = (cw.min(ch - title_h) - 60.0).max(40.0);
                                available / 2.0
                            })
                            .unwrap_or(100.0);

                        // Outer radius for label placement
                        // Reconstruct radius from arc path — use a fixed estimate from arc count
                        // Actually we stored it in layout_chord. Re-derive:

                        view! {
                            <g transform=format!(
                                "translate({cx_off:.2}, {cy_off:.2})",
                            )>
                                // Ribbons (draw first)
                                {lay
                                    .ribbons
                                    .iter()
                                    .map(|r| {
                                        let path = r.path.clone();
                                        let color = r.color.clone();
                                        let value = r.value;
                                        let src = r.source;
                                        let dst = r.target;
                                        let d = data.get();
                                        let src_lbl = d
                                            .labels
                                            .get(src)
                                            .cloned()
                                            .unwrap_or_default();
                                        let dst_lbl = d
                                            .labels
                                            .get(dst)
                                            .cloned()
                                            .unwrap_or_default();
                                        view! {
                                            <path
                                                d=path
                                                fill=color
                                                opacity=ribbon_opacity
                                                style="cursor: default;"
                                                on:mousemove=move |ev| {
                                                    set_hover_label
                                                        .set(Some(format!("{src_lbl} → {dst_lbl}")));
                                                    set_hover_value.set(Some(value));
                                                    set_hover_x.set(ev.offset_x() as f64);
                                                    set_hover_y.set(ev.offset_y() as f64);
                                                }
                                                on:mouseleave=move |_| {
                                                    set_hover_label.set(None);
                                                    set_hover_value.set(None);
                                                }
                                            />
                                        }
                                    })
                                    .collect_view()} // Arcs
                                {lay
                                    .arcs
                                    .iter()
                                    .map(|arc| {
                                        let path = arc.path.clone();
                                        let color = arc.color.clone();
                                        let label = arc.label.clone();
                                        let label2 = label.clone();
                                        let mid = (arc.start_angle + arc.end_angle) / 2.0;
                                        let label_r = outer_r + 14.0;
                                        let lx = outer_r + label_r * mid.sin();
                                        let ly = outer_r - label_r * mid.cos();
                                        let anchor = if mid.sin() >= 0.0 { "start" } else { "end" };
                                        // polar: angle=0 → top, clockwise
                                        view! {
                                            <path
                                                d=path
                                                fill=color
                                                style="cursor: default;"
                                                on:mousemove=move |ev| {
                                                    set_hover_label.set(Some(label.clone()));
                                                    set_hover_value.set(None);
                                                    set_hover_x.set(ev.offset_x() as f64);
                                                    set_hover_y.set(ev.offset_y() as f64);
                                                }
                                                on:mouseleave=move |_| {
                                                    set_hover_label.set(None);
                                                }
                                            />
                                            <text
                                                x=format!("{lx:.2}")
                                                y=format!("{:.2}", ly + th.axis_font_size * 0.35)
                                                text-anchor=anchor
                                                font-size=th.axis_font_size
                                                fill=th.text_color.clone()
                                                pointer-events="none"
                                            >
                                                {label2}
                                            </text>
                                        }
                                    })
                                    .collect_view()} // Tooltip
                                {move || {
                                    let lbl = hover_label.get()?;
                                    let mx = hover_x.get();
                                    let my = hover_y.get();
                                    let val_line = hover_value
                                        .get()
                                        .map(|v| format!("  {v:.1}"))
                                        .unwrap_or_default();
                                    let text = format!("{lbl}{val_line}");
                                    let bw = 12.0 + text.len() as f64 * 6.5;
                                    let bh = 28.0;
                                    let bx = mx - cx_off + 10.0;
                                    let by = my - cy_off + 10.0;
                                    Some(
                                        // Tooltip is relative to the g transform, not SVG root
                                        view! {
                                            <g style="pointer-events: none;">
                                                <rect
                                                    x=format!("{bx:.2}")
                                                    y=format!("{by:.2}")
                                                    width=format!("{bw:.2}")
                                                    height=bh
                                                    rx="4"
                                                    fill="rgba(0,0,0,0.82)"
                                                />
                                                <text
                                                    x=format!("{:.2}", bx + 8.0)
                                                    y=format!("{:.2}", by + 18.0)
                                                    font-size="11"
                                                    fill="white"
                                                    font-family="monospace"
                                                >
                                                    {text}
                                                </text>
                                            </g>
                                        },
                                    )
                                }}
                            </g>
                        }
                    }}
                </svg>
            </div>
        </div>
    }
}

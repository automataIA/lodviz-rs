use crate::hooks::use_container_size;
use leptos::prelude::*;
use lodviz_core::algorithms::beeswarm::beeswarm_layout;
/// Strip chart (dot plot) component
pub use lodviz_core::algorithms::beeswarm::StripLayout;
use lodviz_core::core::data::StripGroup;
use lodviz_core::core::scale::{LinearScale, Scale};
use lodviz_core::core::theme::ChartConfig;

const DEFAULT_PALETTE: &[&str] = &[
    "#4e79a7", "#f28e2b", "#e15759", "#76b7b2", "#59a14f", "#edc948", "#b07aa1", "#ff9da7",
    "#9c755f", "#bab0ac",
];

/// StripChart: categorical groups on the Y axis, individual values scattered on the X axis.
#[component]
pub fn StripChart(
    /// Groups of values to display
    data: Signal<Vec<StripGroup>>,
    /// How to distribute points within a band
    #[prop(default = StripLayout::Jitter)]
    layout: StripLayout,
    /// Circle radius for each data point in SVG units
    #[prop(default = 4.0)]
    point_radius: f64,
    /// Chart configuration
    #[prop(default = Signal::derive(|| ChartConfig::default()), into)]
    config: Signal<ChartConfig>,
    /// Fixed width override
    #[prop(optional)]
    width: Option<u32>,
    /// Fixed height override
    #[prop(optional)]
    height: Option<u32>,
    /// X axis label
    #[prop(optional, into)]
    x_label: Option<String>,
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

    let margin_top = 30.0_f64;
    let margin_bottom = 50.0_f64;
    let margin_left = 90.0_f64;
    let margin_right = 20.0_f64;

    let inner_width =
        Memo::new(move |_| (chart_width.get() as f64 - margin_left - margin_right).max(10.0));
    let inner_height =
        Memo::new(move |_| (chart_height.get() as f64 - margin_top - margin_bottom).max(10.0));

    // X domain: union of all values
    let x_domain = Memo::new(move |_| {
        let groups = data.get();
        let mut xmin = f64::INFINITY;
        let mut xmax = f64::NEG_INFINITY;
        for g in &groups {
            for &v in &g.values {
                if v < xmin {
                    xmin = v;
                }
                if v > xmax {
                    xmax = v;
                }
            }
        }
        if xmin >= xmax {
            xmin -= 1.0;
            xmax += 1.0;
        }
        let pad = (xmax - xmin) * 0.05;
        (xmin - pad, xmax + pad)
    });

    // Hover state for tooltip
    let (hover_group, set_hover_group) = signal(None::<String>);
    let (hover_value, set_hover_value) = signal(None::<f64>);
    let (hover_x, set_hover_x) = signal(0.0_f64);
    let (hover_y, set_hover_y) = signal(0.0_f64);

    let clip_id = format!("strip-clip-{}", uuid::Uuid::new_v4().simple());
    let a11y_title_id = format!("chart-title-{}", uuid::Uuid::new_v4().as_simple());
    let a11y_desc_id = format!("chart-desc-{}", uuid::Uuid::new_v4().as_simple());
    let a11y_labelledby = format!("{} {}", a11y_title_id, a11y_desc_id);

    view! {
        <div
            class="strip-chart"
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
                    <title id=a11y_title_id>{move || final_title.get().unwrap_or("Strip Chart".to_string())}</title>
                    <desc id=a11y_desc_id>"Strip chart showing individual data points per group"</desc>

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

                        // Data points per group
                        {move || {
                            let groups = data.get();
                            if groups.is_empty() {
                                return vec![].into_iter().collect_view();
                            }
                            let n_groups = groups.len();
                            let iw = inner_width.get();
                            let ih = inner_height.get();
                            let band_h = ih / n_groups as f64;
                            let (xmin, xmax) = x_domain.get();
                            let x_scale = LinearScale::new((xmin, xmax), (0.0, iw));
                            let th = theme.get();
                            groups
                                .iter()
                                .enumerate()
                                .map(|(gi, group)| {
                                    let band_center_y = gi as f64 * band_h + band_h / 2.0;
                                    let offsets = beeswarm_layout(
                                        &group.values,
                                        layout,
                                        point_radius,
                                        band_h * 0.85,
                                    );
                                    let color = DEFAULT_PALETTE[gi % DEFAULT_PALETTE.len()]
                                        .to_string();
                                    let gname = group.name.clone();
                                    let circles = group
                                        .values
                                        .iter()
                                        .zip(offsets.iter())
                                        .map(|(&val, &off)| {
                                            let cx = x_scale.map(val);
                                            let cy = band_center_y + off;
                                            let color_c = color.clone();
                                            let gname_c = gname.clone();

                                            view! {
                                                <circle
                                                    cx=format!("{cx:.2}")
                                                    cy=format!("{cy:.2}")
                                                    r=point_radius
                                                    fill=color_c.clone()
                                                    opacity=th.point_opacity
                                                    style="cursor: default;"
                                                    on:mousemove=move |ev| {
                                                        set_hover_group.set(Some(gname_c.clone()));
                                                        set_hover_value.set(Some(val));
                                                        set_hover_x.set(ev.offset_x() as f64 - margin_left);
                                                        set_hover_y.set(ev.offset_y() as f64 - margin_top);
                                                    }
                                                    on:mouseleave=move |_| {
                                                        set_hover_group.set(None);
                                                        set_hover_value.set(None);
                                                    }
                                                />
                                            }
                                        })
                                        .collect_view();
                                    let label = group.name.clone();

                                    // Group label on Y axis
                                    view! {
                                        <g clip-path=format!("url(#{})", clip_id)>{circles}</g>
                                        <text
                                            x="-8"
                                            y=format!("{:.2}", band_center_y + th.axis_font_size * 0.35)
                                            text-anchor="end"
                                            font-size=th.axis_font_size
                                            fill=color.clone()
                                        >
                                            {label}
                                        </text>
                                    }
                                })
                                .collect_view()
                        }}

                        // X axis ticks
                        {move || {
                            let (xmin, xmax) = x_domain.get();
                            let iw = inner_width.get();
                            let ih = inner_height.get();
                            let th = theme.get();
                            let x_scale = LinearScale::new((xmin, xmax), (0.0, iw));
                            let n_ticks = (iw / 80.0).max(2.0) as usize;
                            let ticks: Vec<f64> = (0..=n_ticks)
                                .map(|i| xmin + (xmax - xmin) * i as f64 / n_ticks as f64)
                                .collect();
                            ticks
                                .iter()
                                .map(|&v| {
                                    let tx = x_scale.map(v);
                                    view! {
                                        <g>
                                            <line
                                                x1=format!("{tx:.2}")
                                                y1=format!("{ih:.2}")
                                                x2=format!("{tx:.2}")
                                                y2=format!("{:.2}", ih + 5.0)
                                                stroke=th.axis_color.clone()
                                                stroke-width="1"
                                            />
                                            <text
                                                x=format!("{tx:.2}")
                                                y=format!("{:.2}", ih + 16.0)
                                                text-anchor="middle"
                                                font-size=th.axis_font_size
                                                fill=th.text_color.clone()
                                            >
                                                {format!("{v:.1}")}
                                            </text>
                                        </g>
                                    }
                                })
                                .collect_view()
                        }}

                        // X axis baseline
                        {move || {
                            let iw = inner_width.get();
                            let ih = inner_height.get();
                            let th = theme.get();
                            view! {
                                <line
                                    x1="0"
                                    y1=format!("{ih:.2}")
                                    x2=format!("{iw:.2}")
                                    y2=format!("{ih:.2}")
                                    stroke=th.axis_color.clone()
                                    stroke-width="1"
                                />
                            }
                        }}

                        // X label
                        {x_label
                            .as_ref()
                            .map(|label| {
                                let label = label.clone();
                                move || {
                                    let iw = inner_width.get();
                                    let ih = inner_height.get();
                                    let th = theme.get();
                                    view! {
                                        <text
                                            x=format!("{:.2}", iw / 2.0)
                                            y=format!("{:.2}", ih + 38.0)
                                            text-anchor="middle"
                                            font-size=th.font_size
                                            fill=th.text_color.clone()
                                        >
                                            {label.clone()}
                                        </text>
                                    }
                                }
                            })}

                        // Tooltip
                        {move || {
                            let grp = hover_group.get()?;
                            let val = hover_value.get()?;
                            let mx = hover_x.get();
                            let my = hover_y.get();
                            let iw = inner_width.get();
                            let ih = inner_height.get();
                            let label = format!("{grp}: {val:.3}");
                            let bw = 12.0 + label.len() as f64 * 6.5;
                            let bh = 28.0;
                            let bx = if mx + bw + 12.0 > iw { mx - bw - 12.0 } else { mx + 12.0 };
                            let by = if my + bh + 8.0 > ih { my - bh - 8.0 } else { my + 8.0 };
                            Some(
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
                                            {label}
                                        </text>
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

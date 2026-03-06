/// Sankey flow diagram component
use crate::components::svg::sankey_tooltip::SankeyTooltip;
use crate::hooks::use_container_size;
use leptos::prelude::*;
use lodviz_core::algorithms::sankey_layout::layout_sankey;
use lodviz_core::core::data::SankeyData;
use lodviz_core::core::theme::ChartConfig;

/// SankeyChart: directed flow diagram with node rectangles and ribbon paths.
///
/// Features:
/// - Automatic column assignment via BFS from source nodes
/// - Node heights proportional to flow magnitude
/// - Cubic Bézier ribbons between nodes
/// - Hover tooltip for nodes and ribbons
/// - Responsive layout via `use_container_size()`
#[component]
pub fn SankeyChart(
    /// Sankey data: nodes and directed links
    data: Signal<SankeyData>,
    /// Pixel width of each node rectangle
    #[prop(default = 20.0)]
    node_width: f64,
    /// Vertical gap between nodes in the same column
    #[prop(default = 8.0)]
    node_gap: f64,
    /// Opacity of ribbon (link) paths
    #[prop(default = 0.5)]
    link_opacity: f64,
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
        config.get().width.or(width).unwrap_or(700)
    });
    let chart_height = Memo::new(move |_| {
        let m = container_height.get();
        if m > 0.0 {
            return m as u32;
        }
        config.get().height.or(height).unwrap_or(400)
    });

    let final_title = Memo::new(move |_| config.get().title);

    let margin = 20.0_f64;
    let inner_width = Memo::new(move |_| (chart_width.get() as f64 - margin * 2.0).max(10.0));
    let inner_height =
        Memo::new(move |_| (chart_height.get() as f64 - margin * 2.0 - 30.0).max(10.0));

    // Layout result recomputed when data or dimensions change
    let layout = Memo::new(move |_| {
        let d = data.get();
        let iw = inner_width.get();
        let ih = inner_height.get();
        layout_sankey(&d, iw, ih, node_width, node_gap)
    });

    // Hover state
    let (hover_label, set_hover_label) = signal(None::<String>);
    let (hover_value, set_hover_value) = signal(None::<f64>);
    let (hover_x, set_hover_x) = signal(0.0_f64);
    let (hover_y, set_hover_y) = signal(0.0_f64);

    let iw_signal = Signal::derive(move || inner_width.get());
    let ih_signal = Signal::derive(move || inner_height.get());

    let a11y_title_id = format!("chart-title-{}", uuid::Uuid::new_v4().as_simple());
    let a11y_desc_id = format!("chart-desc-{}", uuid::Uuid::new_v4().as_simple());
    let a11y_labelledby = format!("{} {}", a11y_title_id, a11y_desc_id);

    view! {
        <div
            class="sankey-chart"
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
                        {move || final_title.get().unwrap_or("Sankey diagram".to_string())}
                    </title>
                    <desc id=a11y_desc_id>"Sankey flow diagram showing directional flow between nodes."</desc>

                    <g transform=move || {
                        format!("translate({margin}, {margin})")
                    }>
                        // Ribbons (draw first — behind nodes)
                        {move || {
                            let lay = layout.get();
                            lay.links
                                .iter()
                                .map(|ribbon| {
                                    let path = ribbon.path.clone();
                                    let color = ribbon.color.clone();
                                    let value = ribbon.value;
                                    let src = ribbon.source;
                                    let dst = ribbon.target;
                                    let d = data.get();
                                    let src_label = d
                                        .nodes
                                        .get(src)
                                        .map(|n| n.label.clone())
                                        .unwrap_or_default();
                                    let dst_label = d
                                        .nodes
                                        .get(dst)
                                        .map(|n| n.label.clone())
                                        .unwrap_or_default();
                                    view! {
                                        <path
                                            d=path
                                            fill=color
                                            opacity=link_opacity
                                            style="cursor: default;"
                                            on:mousemove=move |ev| {
                                                set_hover_label
                                                    .set(Some(format!("{src_label} → {dst_label}")));
                                                set_hover_value.set(Some(value));
                                                set_hover_x.set(ev.offset_x() as f64 - margin);
                                                set_hover_y.set(ev.offset_y() as f64 - margin);
                                            }
                                            on:mouseleave=move |_| {
                                                set_hover_label.set(None);
                                                set_hover_value.set(None);
                                            }
                                        />
                                    }
                                })
                                .collect_view()
                        }} // Node rectangles + labels
                        {move || {
                            let lay = layout.get();
                            let th = theme.get();
                            let max_node_x = lay
                                .nodes
                                .iter()
                                .map(|n| (n.x * 1000.0) as i64)
                                .max()
                                .unwrap_or(0);
                            lay.nodes
                                .iter()
                                .map(|node| {
                                    let x = node.x;
                                    let y = node.y;
                                    let w = node.width;
                                    let h = node.height;
                                    let color = node.color.clone();
                                    let label = node.label.clone();
                                    let label2 = label.clone();
                                    let is_last_col = (x * 1000.0) as i64 == max_node_x;
                                    let (text_x, text_anchor) = if is_last_col {
                                        (x - 4.0, "end".to_string())
                                    } else {
                                        (x + w + 4.0, "start".to_string())
                                    };
                                    let node_val_flow = {
                                        let d = data.get();
                                        let idx = node.index;
                                        d
                                            .links
                                            .iter()
                                            .filter(|l| l.source == idx || l.target == idx)
                                            .map(|l| l.value)
                                            .sum::<f64>() / 2.0
                                    };
                                    // Detect last column: nodes with the maximum x value
                                    view! {
                                        <rect
                                            x=format!("{x:.2}")
                                            y=format!("{y:.2}")
                                            width=format!("{w:.2}")
                                            height=format!("{h:.2}")
                                            fill=color.clone()
                                            on:mousemove=move |ev| {
                                                set_hover_label.set(Some(label.clone()));
                                                set_hover_value.set(Some(node_val_flow));
                                                set_hover_x.set(ev.offset_x() as f64 - margin);
                                                set_hover_y.set(ev.offset_y() as f64 - margin);
                                            }
                                            on:mouseleave=move |_| {
                                                set_hover_label.set(None);
                                                set_hover_value.set(None);
                                            }
                                        />
                                        <text
                                            x=format!("{text_x:.2}")
                                            y=format!("{:.2}", y + h / 2.0 + th.axis_font_size * 0.35)
                                            font-size=th.axis_font_size
                                            fill=th.text_color.clone()
                                            text-anchor=text_anchor
                                            pointer-events="none"
                                        >
                                            {label2}
                                        </text>
                                    }
                                })
                                .collect_view()
                        }} // Tooltip
                        <SankeyTooltip
                            label=Signal::derive(move || hover_label.get())
                            value=Signal::derive(move || hover_value.get())
                            x=Signal::derive(move || hover_x.get())
                            y=Signal::derive(move || hover_y.get())
                            inner_width=iw_signal
                            inner_height=ih_signal
                            text_color="white".to_string()
                        />
                    </g>
                </svg>
            </div>
        </div>
    }
}

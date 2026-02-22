use leptos::prelude::*;
use lodviz_core::core::scale::Scale;
use lodviz_core::core::theme::GridStyle;

/// Grid component for rendering background gridlines
///
/// Renders horizontal and/or vertical gridlines based on the scales and style.
#[component]
pub fn Grid<XS: Scale + Clone + 'static, YS: Scale + Clone + 'static>(
    /// X scale for vertical gridlines
    x_scale: XS,
    /// Y scale for horizontal gridlines
    y_scale: YS,
    /// Number of gridlines
    #[prop(default = 5)]
    tick_count: usize,
    /// Width of the grid area
    width: f64,
    /// Height of the grid area
    height: f64,
    /// Grid styling (color, opacity, width, dash, show_x/show_y)
    #[prop(optional)]
    style: Option<GridStyle>,
) -> impl IntoView {
    let gs = style.unwrap_or_default();

    // Vertical gridlines (x-axis ticks)
    let vertical_lines = if gs.show_x {
        let (x_domain_min, x_domain_max) = x_scale.domain();
        let x_tick_values: Vec<f64> = (0..=tick_count)
            .map(|i| {
                let t = i as f64 / tick_count as f64;
                x_domain_min + t * (x_domain_max - x_domain_min)
            })
            .collect();

        let color = gs.color.clone();
        let opacity = gs.opacity;
        let stroke_width = gs.width;
        let dash = gs.dash.clone();

        x_tick_values
            .iter()
            .map(|&value| {
                let x = x_scale.map(value);
                let dash_attr = dash.clone().unwrap_or_default();
                view! {
                    <line
                        key=value
                        x1=x
                        y1=0
                        x2=x
                        y2=height
                        stroke=color.clone()
                        stroke-width=stroke_width
                        opacity=opacity
                        stroke-dasharray=dash_attr
                    />
                }
            })
            .collect_view()
            .into_any()
    } else {
        ().into_any()
    };

    // Horizontal gridlines (y-axis ticks)
    let horizontal_lines = if gs.show_y {
        let (y_domain_min, y_domain_max) = y_scale.domain();
        let y_tick_values: Vec<f64> = (0..=tick_count)
            .map(|i| {
                let t = i as f64 / tick_count as f64;
                y_domain_min + t * (y_domain_max - y_domain_min)
            })
            .collect();

        let color = gs.color.clone();
        let opacity = gs.opacity;
        let stroke_width = gs.width;
        let dash = gs.dash.clone();

        y_tick_values
            .iter()
            .map(|&value| {
                let y = y_scale.map(value);
                let dash_attr = dash.clone().unwrap_or_default();
                view! {
                    <line
                        key=value
                        x1=0
                        y1=y
                        x2=width
                        y2=y
                        stroke=color.clone()
                        stroke-width=stroke_width
                        opacity=opacity
                        stroke-dasharray=dash_attr
                    />
                }
            })
            .collect_view()
            .into_any()
    } else {
        ().into_any()
    };

    view! {
        <g class="grid" pointer-events="none">
            {vertical_lines}
            {horizontal_lines}
        </g>
    }
}

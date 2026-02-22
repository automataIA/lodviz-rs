use leptos::prelude::*;
use leptos::{component, view, IntoView};
/// Axis component for X and Y axes with ticks and labels
use lodviz_core::core::scale::Scale;

/// Axis orientation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AxisOrientation {
    /// Horizontal axis (bottom)
    Bottom,
    /// Horizontal axis (top)
    #[allow(dead_code)]
    Top,
    /// Vertical axis (left)
    Left,
    /// Vertical axis (right)
    #[allow(dead_code)]
    Right,
}

/// Axis component for rendering X/Y axes
///
/// Renders an axis line with ticks and labels based on the provided scale.
/// Optionally renders an axis label (e.g. "Time", "Amplitude").
#[component]
pub fn Axis<S: Scale + Clone + 'static>(
    /// Orientation of the axis
    orientation: AxisOrientation,
    /// Scale for mapping domain to range
    scale: S,
    /// Number of ticks to display
    #[prop(default = 5)]
    tick_count: usize,
    /// Width of the axis (for horizontal) or height (for vertical)
    #[prop(default = 600.0)]
    _dimension: f64,
    /// Stroke color for axis lines and text
    #[prop(default = "currentColor".to_string(), into)]
    stroke: String,
    /// Font size for axis text
    #[prop(default = 12.0)]
    font_size: f64,
    /// Optional axis label text
    #[prop(default = None)]
    label: Option<String>,
) -> impl IntoView {
    let (domain_min, domain_max) = scale.domain();
    let (range_min, range_max) = scale.range();

    // Generate tick values evenly spaced in the domain
    let tick_values: Vec<f64> = (0..=tick_count)
        .map(|i| {
            let t = i as f64 / tick_count as f64;
            domain_min + t * (domain_max - domain_min)
        })
        .collect();

    // Map tick values to positions
    let ticks: Vec<(f64, f64)> = tick_values
        .iter()
        .map(|&value| (value, scale.map(value)))
        .collect();

    let axis_center = (range_min + range_max) / 2.0;

    match orientation {
        AxisOrientation::Bottom => {
            let label_y_offset = 35.0 + font_size * 1.2;
            view! {
                <g class="axis axis-bottom" pointer-events="none">
                    // Axis line
                    <line
                        x1=range_min
                        y1=0
                        x2=range_max
                        y2=0
                        stroke=stroke.clone()
                        stroke-width="1"
                    />

                    // Ticks and labels
                    {ticks
                        .iter()
                        .map(|(value, pos)| {
                            view! {
                                <g key=*value>
                                    // Tick mark
                                    <line
                                        x1=*pos
                                        y1=0
                                        x2=*pos
                                        y2=6
                                        stroke=stroke.clone()
                                        stroke-width="1"
                                    />
                                    // Label
                                    <text
                                        x=*pos
                                        y=20
                                        text-anchor="middle"
                                        font-size=font_size
                                        fill=stroke.clone()
                                    >
                                        {format!("{:.1}", value)}
                                    </text>
                                </g>
                            }
                        })
                        .collect_view()}

                    // Axis label
                    {label
                        .clone()
                        .map(|text| {
                            view! {
                                <text
                                    x=axis_center
                                    y=label_y_offset
                                    text-anchor="middle"
                                    font-size=font_size + 1.0
                                    fill=stroke.clone()
                                >
                                    {text}
                                </text>
                            }
                        })}
                </g>
            }
            .into_any()
        }

        AxisOrientation::Left => {
            let center_y = axis_center;
            let label_x_offset = -(45.0 + font_size * 1.2);
            let ticks_view = ticks
                .iter()
                .map(|(value, pos)| {
                    view! {
                        <g key=*value>
                            <line
                                x1=0
                                y1=*pos
                                x2=-6
                                y2=*pos
                                stroke=stroke.clone()
                                stroke-width="1"
                            />
                            <text
                                x=-10
                                y=*pos
                                text-anchor="end"
                                dominant-baseline="middle"
                                font-size=font_size
                                fill=stroke.clone()
                            >
                                {format!("{:.1}", value)}
                            </text>
                        </g>
                    }
                })
                .collect_view();

            let label_view = label.clone().map(|text| {
                view! {
                    <text
                        transform=format!("rotate(-90, {label_x_offset}, {center_y})")
                        x=label_x_offset
                        y=center_y
                        text-anchor="middle"
                        dominant-baseline="middle"
                        font-size=font_size + 1.0
                        fill=stroke.clone()
                    >
                        {text}
                    </text>
                }
            });

            view! {
                <g class="axis axis-left" pointer-events="none">
                    <line
                        x1=0
                        y1=range_min
                        x2=0
                        y2=range_max
                        stroke=stroke.clone()
                        stroke-width="1"
                    />
                    {ticks_view}
                    {label_view}
                </g>
            }
            .into_any()
        }

        AxisOrientation::Top => view! {
            <g class="axis axis-top" pointer-events="none">
                <line x1=range_min y1=0 x2=range_max y2=0 stroke=stroke.clone() stroke-width="1" />
            </g>
        }
        .into_any(),

        AxisOrientation::Right => view! {
            <g class="axis axis-right" pointer-events="none">
                <line x1=0 y1=range_min x2=0 y2=range_max stroke=stroke.clone() stroke-width="1" />
            </g>
        }
        .into_any(),
    }
}

/// BoxPlot and ViolinChart components
use crate::components::layout::card_registry::get_card_transform_signal;
use crate::components::layout::draggable_card::CardId;
use crate::components::svg::axis::{Axis, AxisOrientation};
use crate::components::svg::box_violin_tooltip::{BoxGroupTooltipData, BoxViolinTooltip};
use crate::components::svg::grid::Grid;
use crate::components::svg::legend::{estimate_legend_width, Legend, LegendItem, LegendPosition};
use crate::hooks::use_container_size;
use leptos::prelude::*;
use lodviz_core::algorithms::statistics::{box_plot_stats, gaussian_kde, BoxPlotStats};
use lodviz_core::core::scale::{BandScale, LinearScale, Scale};
use lodviz_core::core::theme::ChartConfig;

/// Represents a group of raw values grouped by a single category label.
///
/// This is used as the main data structure for the [`BoxPlot`] and [`ViolinChart`] components.
/// Statistical calculations (such as Kernel Density Estimation or quartiles) are
/// performed automatically by the component starting from this structure.
///
/// If the `data` vector is empty, the group will be excluded from calculation and the UI will adapt accordingly.
#[derive(Clone, Debug)]
pub struct BoxGroup {
    /// The label of the category (e.g., "Group A")
    pub label: String,
    /// The raw numerical data points for this category
    pub data: Vec<f64>,
}

/// Computed per-group data (pixel coordinates already applied)
#[derive(Clone, Debug)]
struct GroupLayout {
    center: f64,
    band_width: f64,
    stats: BoxPlotStats,
}

/// Compute group layouts from BoxGroup data and scales.
fn compute_groups(
    groups: &[BoxGroup],
    x_band: &BandScale,
    y_scale: &LinearScale,
) -> Vec<(GroupLayout, Vec<(f64, f64)>)> {
    groups
        .iter()
        .enumerate()
        .filter_map(|(i, g)| {
            if g.data.is_empty() {
                return None;
            }
            let mut data_copy = g.data.clone();
            let stats = box_plot_stats(&mut data_copy)?;
            let center = x_band.map_index_center(i);
            let bw = x_band.band_width();
            let layout = GroupLayout {
                center,
                band_width: bw,
                stats: stats.clone(),
            };
            // Map outlier pixel positions
            let outlier_pxs: Vec<(f64, f64)> = stats
                .outliers
                .iter()
                .map(|&v| (center, y_scale.map(v)))
                .collect();
            Some((layout, outlier_pxs))
        })
        .collect()
}

/// A responsive and interactive Box Plot chart for visualizing statistical distributions.
///
/// This chart displays the median, first and third quartiles (IQR), whiskers,
/// and any outlier values for each provided [`BoxGroup`].
///
/// # Layout and Dimensions
/// By default, the chart attempts to fill its container using responsive dimensions.
/// If placed inside a [`DraggableCard`](crate::components::layout::draggable_card::DraggableCard),
/// it will automatically adapt to the card's dimensions.
/// Alternatively, you can force a specific width and height by overriding the `config` prop.
///
/// # Panics
///
/// This component does not intentionally panic. Statistical calculations on empty groups
/// are silently ignored.
///
/// # Examples
///
/// ```rust,ignore
/// use lodviz_components::components::charts::box_plot::{BoxGroup, BoxPlot};
/// use lodviz_core::core::theme::ChartConfig;
/// use leptos::prelude::*;
///
/// #[component]
/// fn MyBoxPlot() -> impl IntoView {
///     let data = vec![
///         BoxGroup { label: "Group A".into(), data: vec![1.2, 2.5, 3.1, 4.0, 5.0, 10.0] },
///         BoxGroup { label: "Group B".into(), data: vec![0.5, 1.0, 1.5, 2.0, 2.5] },
///     ];
///
///     view! {
///         <BoxPlot
///             data=Signal::derive(move || data.clone())
///             config=Signal::derive(|| ChartConfig::default())
///             y_label=Some("Value".to_string())
///         />
///     }
/// }
/// ```
#[component]
pub fn BoxPlot(
    /// Groups of raw data values with labels
    data: Signal<Vec<BoxGroup>>,
    /// Chart configuration (title, theme, margin …)
    #[prop(default = Signal::derive(|| ChartConfig::default()), into)]
    config: Signal<ChartConfig>,
    /// Fixed width (falls back to card / default)
    #[prop(optional)]
    width: Option<u32>,
    /// Fixed height (falls back to card / default)
    #[prop(optional)]
    height: Option<u32>,
    /// Y axis label
    #[prop(optional, into)]
    y_label: Option<String>,
) -> impl IntoView {
    let theme = Memo::new(move |_| config.get().theme.unwrap_or_default());

    let (container_width, container_height, container_ref) = use_container_size();
    let card_transform = use_context::<CardId>().map(|id| get_card_transform_signal(id.0.clone()));

    let chart_width = Memo::new(move |_| {
        let measured = container_width.get();
        if measured > 0.0 {
            return measured as u32;
        }
        config
            .get()
            .width
            .or(width)
            .or_else(|| {
                card_transform.and_then(|sig| sig.get().map(|ct| (ct.width - 32.0).max(0.0) as u32))
            })
            .unwrap_or(800)
    });
    let chart_height = Memo::new(move |_| {
        let measured = container_height.get();
        if measured > 0.0 {
            return measured as u32;
        }
        config
            .get()
            .height
            .or(height)
            .or_else(|| {
                card_transform
                    .and_then(|sig| sig.get().map(|ct| (ct.height - 40.0).max(100.0) as u32))
            })
            .unwrap_or(400)
    });

    // Legend items — defined early so margin can adapt when legend_outside is enabled
    let legend_items = Signal::derive(move || {
        let groups = data.get();
        let th = theme.get();
        groups
            .iter()
            .enumerate()
            .map(|(i, g)| LegendItem {
                name: g.label.clone(),
                color: th.palette[i % th.palette.len()].clone(),
                visible: true,
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

    let final_title = Memo::new(move |_| config.get().title.clone());

    // Y scale: extent over ALL groups including whiskers + 5% padding
    let y_scale = Memo::new(move |_| {
        let groups = data.get();
        let mut all_vals: Vec<f64> = groups.iter().flat_map(|g| g.data.iter().copied()).collect();
        if all_vals.is_empty() {
            all_vals = vec![0.0, 1.0];
        }
        all_vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let y_min = all_vals[0];
        let y_max = all_vals[all_vals.len() - 1];
        let pad = (y_max - y_min) * 0.1;
        LinearScale::new((y_min - pad, y_max + pad), (inner_height.get(), 0.0))
    });

    let y_tick_count = Memo::new(move |_| (inner_height.get() / 50.0).max(2.0) as usize);

    let y_label_clone = y_label.clone();

    // Memoized band scale (padding=0.3 for BoxPlot)
    let x_band_memo = Memo::new(move |_| {
        let groups = data.get();
        BandScale::new(
            groups.iter().map(|g| g.label.clone()).collect(),
            (0.0, inner_width.get()),
            0.3,
        )
    });

    let show_legend = Memo::new(move |_| {
        config
            .get()
            .show_legend
            .unwrap_or_else(|| legend_items.get().len() > 1)
    });

    // Tooltip data memo (one entry per group)
    let groups_tooltip: Memo<Vec<BoxGroupTooltipData>> = Memo::new(move |_| {
        let groups = data.get();
        let x_band = x_band_memo.get();
        let th = theme.get();
        groups
            .iter()
            .enumerate()
            .filter_map(|(i, g)| {
                if g.data.is_empty() {
                    return None;
                }
                let mut data_copy = g.data.clone();
                let stats = box_plot_stats(&mut data_copy)?;
                let center = x_band.map_index_center(i);
                let bw = x_band.band_width();
                let color = th.palette[i % th.palette.len()].clone();
                Some(BoxGroupTooltipData {
                    label: g.label.clone(),
                    center,
                    band_width: bw,
                    stats,
                    n: g.data.len(),
                    color,
                })
            })
            .collect()
    });

    view! {
        <div
            class="box-plot"
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
                    aria-label="Box plot chart"
                    viewBox=move || format!("0 0 {} {}", chart_width.get(), chart_height.get())
                    style="width: 100%; height: 100%; display: block;"
                >
                    <g transform=move || {
                        format!("translate({}, {})", margin.get().left, margin.get().top)
                    }>
                        // Grid (Y only for box plots)
                        {move || {
                            let ys = y_scale.get();
                            let dummy_xs = LinearScale::new(
                                (0.0, inner_width.get()),
                                (0.0, inner_width.get()),
                            );
                            view! {
                                <Grid
                                    x_scale=dummy_xs
                                    y_scale=ys
                                    tick_count=y_tick_count.get()
                                    width=inner_width.get()
                                    height=inner_height.get()
                                    style=theme.get().grid.clone()
                                />
                            }
                        }} // Boxes, whiskers, outliers
                        {move || {
                            let groups = data.get();
                            let ys = y_scale.get();
                            let th = theme.get();
                            let iw = inner_width.get();
                            let x_band = BandScale::new(
                                groups.iter().map(|g| g.label.clone()).collect(),
                                (0.0, iw),
                                0.3,
                            );
                            let computed = compute_groups(&groups, &x_band, &ys);
                            computed
                                .iter()
                                .enumerate()
                                .map(|(i, (gl, outlier_pxs))| {
                                    let color = th.palette[i % th.palette.len()].clone();
                                    let s = &gl.stats;
                                    let cx = gl.center;
                                    let bw = gl.band_width;
                                    let half_bw = bw / 2.0;
                                    let quarter_bw = bw / 4.0;
                                    let y_q1 = ys.map(s.q1);
                                    let y_q3 = ys.map(s.q3);
                                    let y_med = ys.map(s.median);
                                    let y_lw = ys.map(s.lower_whisker);
                                    let y_uw = ys.map(s.upper_whisker);
                                    let box_x = cx - half_bw;
                                    let box_y = y_q3.min(y_q1);
                                    let box_h = (y_q1 - y_q3).abs();
                                    let outlier_views: Vec<_> = outlier_pxs
                                        .iter()
                                        .map(|&(ox, oy)| {

                                            view! {
                                                <circle
                                                    cx=format!("{ox:.2}")
                                                    cy=format!("{oy:.2}")
                                                    r=3
                                                    fill="none"
                                                    stroke=color.clone()
                                                    stroke-width=1.5
                                                />
                                            }
                                        })
                                        .collect();

                                    view! {
                                        <g>
                                            // Lower whisker stem
                                            <line
                                                x1=format!("{cx:.2}")
                                                y1=format!("{y_q1:.2}")
                                                x2=format!("{cx:.2}")
                                                y2=format!("{y_lw:.2}")
                                                stroke=color.clone()
                                                stroke-width=1.5
                                                stroke-dasharray="4,2"
                                            />
                                            // Upper whisker stem
                                            <line
                                                x1=format!("{cx:.2}")
                                                y1=format!("{y_q3:.2}")
                                                x2=format!("{cx:.2}")
                                                y2=format!("{y_uw:.2}")
                                                stroke=color.clone()
                                                stroke-width=1.5
                                                stroke-dasharray="4,2"
                                            />
                                            // Lower T-bar
                                            <line
                                                x1=format!("{:.2}", cx - quarter_bw)
                                                y1=format!("{y_lw:.2}")
                                                x2=format!("{:.2}", cx + quarter_bw)
                                                y2=format!("{y_lw:.2}")
                                                stroke=color.clone()
                                                stroke-width=2
                                            />
                                            // Upper T-bar
                                            <line
                                                x1=format!("{:.2}", cx - quarter_bw)
                                                y1=format!("{y_uw:.2}")
                                                x2=format!("{:.2}", cx + quarter_bw)
                                                y2=format!("{y_uw:.2}")
                                                stroke=color.clone()
                                                stroke-width=2
                                            />
                                            // IQR box
                                            <rect
                                                x=format!("{box_x:.2}")
                                                y=format!("{box_y:.2}")
                                                width=format!("{bw:.2}")
                                                height=format!("{box_h:.2}")
                                                fill=format!("{}33", color)
                                                stroke=color.clone()
                                                stroke-width=2
                                            />
                                            // Median line
                                            <line
                                                x1=format!("{:.2}", cx - half_bw)
                                                y1=format!("{y_med:.2}")
                                                x2=format!("{:.2}", cx + half_bw)
                                                y2=format!("{y_med:.2}")
                                                stroke=color.clone()
                                                stroke-width=3
                                            />
                                            // Outliers
                                            {outlier_views}
                                        </g>
                                    }
                                })
                                .collect_view()
                        }} // Category labels on X axis
                        {move || {
                            let groups = data.get();
                            let th = theme.get();
                            let x_band = BandScale::new(
                                groups.iter().map(|g| g.label.clone()).collect(),
                                (0.0, inner_width.get()),
                                0.3,
                            );
                            groups
                                .iter()
                                .enumerate()
                                .map(|(i, g)| {
                                    let cx = x_band.map_index_center(i);
                                    let ty = inner_height.get() + 18.0;
                                    view! {
                                        <text
                                            x=format!("{cx:.2}")
                                            y=format!("{ty:.2}")
                                            text-anchor="middle"
                                            font-size=th.axis_font_size
                                            fill=th.axis_color.clone()
                                        >
                                            {g.label.clone()}
                                        </text>
                                    }
                                })
                                .collect_view()
                        }} // Y axis
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
                        }} // Tooltip overlay (last = captures mouse events above chart content)
                        <BoxViolinTooltip
                            groups=groups_tooltip
                            band_scale=x_band_memo
                            inner_width=inner_width
                            inner_height=inner_height
                            margin=margin
                        /> // SVG Legend overlay (must be last to render on top)
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

/// A Violin Chart that combines probability densities with an internal mini box-plot.
///
/// Unlike a standard [`BoxPlot`], the violin chart automatically calculates
/// a Kernel Density Estimation (KDE) to interpolate the shape of the distribution curve for
/// each `BoxGroup`. The props and layout behavior are identical to the `BoxPlot`.
///
/// # Computation
/// The violin curve is generated by calling [`gaussian_kde`](lodviz_core::algorithms::statistics::gaussian_kde)
/// on the raw data, sampling 80 points. If the sample has fewer than 2 elements, the KDE calculation
/// is invalid and the group is skipped visually.
///
/// # Examples
///
/// ```rust,ignore
/// use lodviz_components::components::charts::box_plot::{BoxGroup, ViolinChart};
/// use lodviz_core::core::theme::ChartConfig;
/// use leptos::prelude::*;
///
/// #[component]
/// fn MyViolin() -> impl IntoView {
///     let data = vec![
///         BoxGroup { label: "Exposure 1".into(), data: vec![.../* real data */] },
///     ];
///     view! {
///         <ViolinChart data=Signal::derive(move || data.clone()) />
///     }
/// }
/// ```
#[component]
pub fn ViolinChart(
    /// Groups of raw data values with labels
    data: Signal<Vec<BoxGroup>>,
    /// Chart configuration (title, theme, margin …)
    #[prop(default = Signal::derive(|| ChartConfig::default()), into)]
    config: Signal<ChartConfig>,
    /// Fixed width
    #[prop(optional)]
    width: Option<u32>,
    /// Fixed height
    #[prop(optional)]
    height: Option<u32>,
    /// Y axis label
    #[prop(optional, into)]
    y_label: Option<String>,
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

    // Legend items — defined early so margin can adapt when legend_outside is enabled
    let legend_items = Signal::derive(move || {
        let groups = data.get();
        let th = theme.get();
        groups
            .iter()
            .enumerate()
            .map(|(i, g)| LegendItem {
                name: g.label.clone(),
                color: th.palette[i % th.palette.len()].clone(),
                visible: true,
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

    let final_title = Memo::new(move |_| config.get().title.clone());

    // Pre-calculate KDEs for all groups to determine global Y extent
    // We store (KDE, max_density) because max_density is needed for width scaling
    // Using Signal::derive instead of Memo because KdeResult doesn't implement PartialEq
    let violin_kdes = Signal::derive(move || {
        data.get()
            .iter()
            .map(|g| {
                if g.data.len() < 2 {
                    return None;
                }
                let kde = gaussian_kde(&g.data, 80)?;
                let max_density = kde.ys.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                if max_density <= 0.0 {
                    None
                } else {
                    Some((kde, max_density))
                }
            })
            .collect::<Vec<_>>()
    });

    let y_scale = Memo::new(move |_| {
        // Calculate extent from the generated KDE points, NOT just raw data
        // This prevents the tails of the Gaussian distribution from being clipped
        let kdes = violin_kdes.get();
        let mut min_val = f64::INFINITY;
        let mut max_val = f64::NEG_INFINITY;
        let mut has_data = false;

        for (kde, _) in kdes.into_iter().flatten() {
            if let Some(min) = kde.xs.first() {
                min_val = min_val.min(*min);
            }
            if let Some(max) = kde.xs.last() {
                max_val = max_val.max(*max);
            }
            has_data = true;
        }

        if !has_data {
            min_val = 0.0;
            max_val = 1.0;
        }

        // Add small padding
        let pad = (max_val - min_val) * 0.05;
        LinearScale::new((min_val - pad, max_val + pad), (inner_height.get(), 0.0))
    });

    let y_tick_count = Memo::new(move |_| (inner_height.get() / 50.0).max(2.0) as usize);
    let y_label_clone = y_label.clone();

    // Memoized band scale (padding=0.25 for ViolinChart)
    let x_band_memo_violin = Memo::new(move |_| {
        let groups = data.get();
        BandScale::new(
            groups.iter().map(|g| g.label.clone()).collect(),
            (0.0, inner_width.get()),
            0.25,
        )
    });

    // Tooltip data memo (one entry per group, same stats as box plot)
    let groups_tooltip_violin: Memo<Vec<BoxGroupTooltipData>> = Memo::new(move |_| {
        let groups = data.get();
        let x_band = x_band_memo_violin.get();
        let th = theme.get();
        groups
            .iter()
            .enumerate()
            .filter_map(|(i, g)| {
                if g.data.is_empty() {
                    return None;
                }
                let mut data_copy = g.data.clone();
                let stats = box_plot_stats(&mut data_copy)?;
                let center = x_band.map_index_center(i);
                let bw = x_band.band_width();
                let color = th.palette[i % th.palette.len()].clone();
                Some(BoxGroupTooltipData {
                    label: g.label.clone(),
                    center,
                    band_width: bw,
                    stats,
                    n: g.data.len(),
                    color,
                })
            })
            .collect()
    });

    let show_legend = Memo::new(move |_| {
        config
            .get()
            .show_legend
            .unwrap_or_else(|| legend_items.get().len() > 1)
    });

    view! {
        <div
            class="violin-chart"
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
                    aria-label="Violin chart"
                    viewBox=move || format!("0 0 {} {}", chart_width.get(), chart_height.get())
                    style="width: 100%; height: 100%; display: block;"
                >
                    <g transform=move || {
                        format!("translate({}, {})", margin.get().left, margin.get().top)
                    }>
                        // Grid (Y only)
                        {move || {
                            let ys = y_scale.get();
                            let dummy_xs = LinearScale::new(
                                (0.0, inner_width.get()),
                                (0.0, inner_width.get()),
                            );
                            view! {
                                <Grid
                                    x_scale=dummy_xs
                                    y_scale=ys
                                    tick_count=y_tick_count.get()
                                    width=inner_width.get()
                                    height=inner_height.get()
                                    style=theme.get().grid.clone()
                                />
                            }
                        }} // Violin shapes + box overlay
                        {move || {
                            let groups = data.get();
                            let kdes = violin_kdes.get();
                            let ys = y_scale.get();
                            let th = theme.get();
                            let iw = inner_width.get();
                            let x_band = BandScale::new(
                                groups.iter().map(|g| g.label.clone()).collect(),
                                (0.0, iw),
                                0.25,
                            );
                            groups
                                .iter()
                                .zip(kdes.iter())
                                .enumerate()
                                .filter_map(|(i, (g, kde_opt))| {
                                    let (kde, max_density) = kde_opt.as_ref()?;
                                    let color = th.palette[i % th.palette.len()].clone();
                                    let cx = x_band.map_index_center(i);
                                    let bw = x_band.band_width();
                                    let max_half_width = bw / 2.0 * 0.9;
                                    let n = kde.xs.len();
                                    let mut right: Vec<(f64, f64)> = Vec::with_capacity(n);
                                    let mut left: Vec<(f64, f64)> = Vec::with_capacity(n);
                                    for j in 0..n {
                                        let y_px = ys.map(kde.xs[j]);
                                        let w = (kde.ys[j] / max_density) * max_half_width;
                                        right.push((cx + w, y_px));
                                        left.push((cx - w, y_px));
                                    }
                                    let mut path = String::new();
                                    for (k, &(px, py)) in right.iter().enumerate() {
                                        if k == 0 {
                                            path.push_str(&format!("M {px:.2} {py:.2}"));
                                        } else {
                                            path.push_str(&format!(" L {px:.2} {py:.2}"));
                                        }
                                    }
                                    for &(px, py) in left.iter().rev() {
                                        path.push_str(&format!(" L {px:.2} {py:.2}"));
                                    }
                                    path.push_str(" Z");
                                    let mut data_copy = g.data.clone();
                                    let stats = box_plot_stats(&mut data_copy)?;
                                    let y_q1 = ys.map(stats.q1);
                                    let y_q3 = ys.map(stats.q3);
                                    let y_med = ys.map(stats.median);
                                    let box_half = bw * 0.12;
                                    Some(
                                        // Use pre-calculated KDE

                                        // KDE

                                        // Build violin path

                                        // Box stats overlay

                                        view! {
                                            <g>
                                                // Violin shape
                                                <path
                                                    d=path
                                                    fill=format!("{}55", color)
                                                    stroke=color.clone()
                                                    stroke-width=1.5
                                                />
                                                // IQR box overlay
                                                <rect
                                                    x=format!("{:.2}", cx - box_half)
                                                    y=format!("{:.2}", y_q3.min(y_q1))
                                                    width=format!("{:.2}", box_half * 2.0)
                                                    height=format!("{:.2}", (y_q1 - y_q3).abs())
                                                    fill="#ffffff"
                                                    stroke=color.clone()
                                                    stroke-width=2
                                                />
                                                // Median dot
                                                <circle
                                                    cx=format!("{cx:.2}")
                                                    cy=format!("{y_med:.2}")
                                                    r=4
                                                    fill=color.clone()
                                                />
                                            </g>
                                        },
                                    )
                                })
                                .collect_view()
                        }} // Category labels
                        {move || {
                            let groups = data.get();
                            let th = theme.get();
                            let x_band = BandScale::new(
                                groups.iter().map(|g| g.label.clone()).collect(),
                                (0.0, inner_width.get()),
                                0.25,
                            );
                            groups
                                .iter()
                                .enumerate()
                                .map(|(i, g)| {
                                    let cx = x_band.map_index_center(i);
                                    let ty = inner_height.get() + 18.0;
                                    view! {
                                        <text
                                            x=format!("{cx:.2}")
                                            y=format!("{ty:.2}")
                                            text-anchor="middle"
                                            font-size=th.axis_font_size
                                            fill=th.axis_color.clone()
                                        >
                                            {g.label.clone()}
                                        </text>
                                    }
                                })
                                .collect_view()
                        }} // Y axis
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
                        }} // Tooltip overlay (last = captures mouse events above chart content)
                        <BoxViolinTooltip
                            groups=groups_tooltip_violin
                            band_scale=x_band_memo_violin
                            inner_width=inner_width
                            inner_height=inner_height
                            margin=margin
                        /> // SVG Legend overlay (must be last to render on top)
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

/// SmartChart — facade component that dispatches to the correct chart renderer.
///
/// Takes a `ChartSpec` (which can contain a raw `DataTable`, a `Dataset`, or a
/// `BarDataset`) and renders the appropriate chart based on the `Mark` type.
///
/// Conversion from `DataTable` to the concrete dataset type happens lazily here,
/// using the `x`, `y`, and `color` fields from the spec as column selectors.
use crate::components::charts::area_chart::AreaChart;
use crate::components::charts::bar_chart::{BarChart, BarMode, BarOrientation};
use crate::components::charts::line_chart::LineChart;
use crate::components::charts::scatter_chart::ScatterChart;
use leptos::prelude::*;
use lodviz_core::core::data::{BarDataset, Dataset};
use lodviz_core::core::encoding::Encoding;
use lodviz_core::core::mark::Mark;
use lodviz_core::core::spec::{ChartData, ChartSpec};
use lodviz_core::core::theme::ChartConfig;

// --- Helpers: resolve ChartData → concrete dataset types ---

/// Resolve the spec's `ChartData` to a `Dataset`.
///
/// - `ChartData::TimeSeries(ds)` → used directly
/// - `ChartData::Table(table)` → converted via `to_dataset()` using spec encoding
/// - `ChartData::Categorical(_)` → returns empty Dataset
fn resolve_dataset(spec: &ChartSpec) -> Dataset {
    match &spec.data {
        ChartData::TimeSeries(ds) => ds.clone(),
        ChartData::Table(table) => {
            let Some(y_field) = &spec.y else {
                return Dataset::new();
            };
            let enc = Encoding::new(spec.x.clone(), y_field.clone())
                .with_color_opt(spec.color.clone())
                .with_size_opt(spec.size.clone());
            table.to_dataset(&enc)
        }
        ChartData::Categorical(_) => Dataset::new(),
    }
}

/// Resolve the spec's `ChartData` to a `BarDataset`.
///
/// - `ChartData::Categorical(bd)` → used directly
/// - `ChartData::Table(table)` → converted via `to_bar_dataset()` using spec encoding
/// - `ChartData::TimeSeries(_)` → returns empty BarDataset
fn resolve_bar_dataset(spec: &ChartSpec) -> BarDataset {
    match &spec.data {
        ChartData::Categorical(bd) => bd.clone(),
        ChartData::Table(table) => {
            let Some(y_field) = &spec.y else {
                return BarDataset::new(vec![]);
            };
            let enc =
                Encoding::new(spec.x.clone(), y_field.clone()).with_color_opt(spec.color.clone());
            table.to_bar_dataset(&enc)
        }
        ChartData::TimeSeries(_) => BarDataset::new(vec![]),
    }
}

// --- SmartChart component ---

/// Facade chart component that picks the right renderer from a `ChartSpec`.
///
/// # Usage
///
/// ```rust,ignore
/// let spec = ChartSpec::builder()
///     .from_table(DataTable::from_rows(rows))
///     .mark(Mark::Line)
///     .x(Field::temporal("date"))
///     .y(Field::quantitative("amount"))
///     .color(Field::nominal("product"))
///     .title("Sales Trend")
///     .build();
///
/// view! { <SmartChart spec=Signal::derive(move || spec.clone()) /> }
/// ```
///
/// # Supported marks
/// | `Mark`          | Component      |
/// |-----------------|----------------|
/// | `Line`          | `LineChart`    |
/// | `Area`          | `AreaChart`    |
/// | `Bar`           | `BarChart`     |
/// | `Point`/`Circle`| `ScatterChart` |
#[component]
pub fn SmartChart(
    /// Reactive chart specification (mark + encoding + data)
    spec: Signal<ChartSpec>,
    /// Override width in pixels (merged into config; optional)
    #[prop(optional)]
    width: Option<u32>,
    /// Override height in pixels (merged into config; optional)
    #[prop(optional)]
    height: Option<u32>,
    /// Parent card ID for reactive dimensions (optional, reads from CardId context if not provided)
    #[prop(optional)]
    card_id: Option<String>,
    /// Override chart config (uses spec.config as baseline if default)
    #[prop(default = Signal::derive(|| ChartConfig::default()), into)]
    config: Signal<ChartConfig>,
) -> impl IntoView {
    use crate::components::layout::card_registry::get_card_transform_signal;
    use crate::components::layout::draggable_card::CardId;

    // Try to get card_id from prop or context
    let effective_card_id = Signal::derive(move || {
        if let Some(ref id) = card_id {
            return Some(id.clone());
        }
        // Try to read from context (fallback)
        use_context::<CardId>().map(|cid| cid.0)
    });

    // Merged config: spec.config baseline, overridden by external config and card dimensions
    let resolved_config = Signal::derive(move || {
        let s = spec.get();
        let external = config.get();
        let mut cfg = if external.title.is_some() || external.theme.is_some() {
            external
        } else {
            s.config.clone()
        };

        // Card dimensions from registry (reactive) take precedence over width/height props
        if let Some(id) = effective_card_id.get() {
            if let Some(transform) = get_card_transform_signal(id).get() {
                // Subtract padding: 32px for width, 40px for height
                cfg.width = Some((transform.width - 32.0).max(0.0) as u32);
                cfg.height = Some((transform.height - 40.0).max(100.0) as u32);
            }
        }

        // width/height props override config (but card dimensions take precedence above)
        if cfg.width.is_none() {
            if let Some(w) = width {
                cfg.width = Some(w);
            }
        }
        if cfg.height.is_none() {
            if let Some(h) = height {
                cfg.height = Some(h);
            }
        }

        cfg
    });

    // Signal::derive avoids PartialEq requirement (vs Memo::new)
    let dataset = Signal::derive(move || resolve_dataset(&spec.get()));
    let bar_dataset = Signal::derive(move || resolve_bar_dataset(&spec.get()));

    let mark = Memo::new(move |_| spec.get().mark);

    view! {
        <div style="width: 100%; height: 100%;">
            {move || match mark.get() {
                Mark::Line => view! { <LineChart data=dataset config=resolved_config /> }.into_any(),
                Mark::Area => view! { <AreaChart data=dataset config=resolved_config /> }.into_any(),
                Mark::Bar => {
                    view! {
                        <BarChart
                            data=bar_dataset
                            orientation=BarOrientation::Vertical
                            mode=BarMode::Grouped
                            config=resolved_config
                        />
                    }
                        .into_any()
                }
                Mark::Point | Mark::Circle => {
                    view! { <ScatterChart data=dataset config=resolved_config /> }.into_any()
                }
            }}
        </div>
    }
}

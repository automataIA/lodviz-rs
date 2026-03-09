use crate::BASE_PATH;
use crate::components::ui::navbar::Navbar;
use crate::components::ui::theme_editor::ThemeEditor;
use crate::data::csv_loader::{
    fetch_csv, to_box_groups, to_grouped_bar, to_histogram_values, to_multi_dataset, to_ohlc,
    to_pie_entries, to_radar_series, to_waterfall_bars, to_xy_dataset,
};
use leptos::prelude::*;
use lodviz_components::DataTable;
use lodviz_components::components::charts::area_chart::AreaChart;
use lodviz_components::components::charts::bar_chart::{BarChart, BarMode};
use lodviz_components::components::charts::box_plot::{BoxPlot, ViolinChart};
use lodviz_components::components::charts::candlestick::CandlestickChart;
use lodviz_components::components::charts::chord_chart::ChordChart;
use lodviz_components::components::charts::contour_chart::ContourChart;
use lodviz_components::components::charts::heatmap::HeatmapChart;
use lodviz_components::components::charts::histogram::Histogram;
use lodviz_components::components::charts::line_chart::LineChart;
use lodviz_components::components::charts::pie_chart::PieChart;
use lodviz_components::components::charts::radar::RadarChart;
use lodviz_components::components::charts::sankey_chart::SankeyChart;
use lodviz_components::components::charts::scatter_chart::ScatterChart;
use lodviz_components::components::charts::smart_chart::SmartChart;
use lodviz_components::components::charts::strip_chart::{StripChart, StripLayout};
use lodviz_components::components::charts::waterfall::WaterfallChart;
use lodviz_components::components::interaction::linked_context::LinkedDashboard;
use lodviz_components::components::layout::chart_visibility::ChartVisibility;
use lodviz_components::components::layout::draggable_card::{CardTransform, DraggableCard};
use lodviz_components::components::theme_provider::ThemeProvider;
use lodviz_core::core::color_map::{ColorMap, SequentialColorMap};
use lodviz_core::core::data::{
    BarDataset, ChordData, DataPoint, Dataset, GridData, SankeyData, SankeyLink, SankeyNode,
    Series, StripGroup,
};
use lodviz_core::core::encoding::Field;
use lodviz_core::core::mark::Mark;
use lodviz_core::core::spec::ChartSpec;
use lodviz_core::core::table_data::{Alignment, ColumnType, TableData};
use lodviz_core::core::theme::ColorScheme;
use lodviz_core::core::theme::{ChartConfig, GridStyle};

/// Home Page - lodviz-rs Demo
#[component]
pub fn Home() -> impl IntoView {
    // --- Data signals + background loaders ---
    //
    // Pattern: RwSignal<T> initialized to empty/default, then populated by LocalResource.
    // The chart component is ALWAYS in the DOM from initial render so it can inherit
    // DraggableCard's provide_context(Signal<CardTransform>). Data arrives reactively
    // when the HTTP fetch completes.

    // Demo 1: Sine Wave
    let sine_data = RwSignal::new(Dataset::new());
    let _sine_res = LocalResource::new(move || async move {
        match fetch_csv(&format!("{BASE_PATH}/data/sine_wave.csv")).await {
            Ok(table) => sine_data.set(to_xy_dataset(&table, "x", "y", "Sine")),
            Err(e) => log::error!("sine_wave.csv: {e}"),
        }
    });

    // Demo 2: Large Dataset (5000 points — LTTB demo)
    let large_data = RwSignal::new(Dataset::new());
    let _large_res = LocalResource::new(move || async move {
        match fetch_csv(&format!("{BASE_PATH}/data/large_dataset.csv")).await {
            Ok(table) => large_data.set(to_xy_dataset(&table, "x", "y", "Signal")),
            Err(e) => log::error!("large_dataset.csv: {e}"),
        }
    });

    // Demo 3: Area Chart (multi-series)
    let area_data = RwSignal::new(Dataset::new());
    let _area_res = LocalResource::new(move || async move {
        match fetch_csv(&format!("{BASE_PATH}/data/area_chart.csv")).await {
            Ok(table) => area_data.set(to_multi_dataset(
                &table,
                "x",
                &[("revenue", "Revenue"), ("costs", "Costs")],
            )),
            Err(e) => log::error!("area_chart.csv: {e}"),
        }
    });

    // Demo 4: Scatter Plot
    let scatter_data = RwSignal::new(Dataset::new());
    let _scatter_res = LocalResource::new(move || async move {
        match fetch_csv(&format!("{BASE_PATH}/data/scatter.csv")).await {
            Ok(table) => scatter_data.set(to_xy_dataset(&table, "x", "y", "Correlation")),
            Err(e) => log::error!("scatter.csv: {e}"),
        }
    });

    // Demo 5: Multi-series (sin, cos, tan) — RwSignal needed for visibility toggle
    let multi_data = RwSignal::new(Dataset::new());
    let _multi_res = LocalResource::new(move || async move {
        match fetch_csv(&format!("{BASE_PATH}/data/multi_series.csv")).await {
            Ok(table) => multi_data.set(to_multi_dataset(
                &table,
                "x",
                &[
                    ("sin_x", "sin(x)"),
                    ("cos_x", "cos(x)"),
                    ("tan_x", "tan(x)"),
                ],
            )),
            Err(e) => log::error!("multi_series.csv: {e}"),
        }
    });

    // Demo 6: Bar Chart (grouped)
    let bar_data = RwSignal::new(BarDataset::default());
    let _bar_res = LocalResource::new(move || async move {
        match fetch_csv(&format!("{BASE_PATH}/data/bar_grouped.csv")).await {
            Ok(table) => bar_data.set(to_grouped_bar(
                &table,
                "category",
                &[
                    ("product_a", "Product A"),
                    ("product_b", "Product B"),
                    ("product_c", "Product C"),
                ],
            )),
            Err(e) => log::error!("bar_grouped.csv: {e}"),
        }
    });

    // Demo 7: Bar Chart (stacked)
    let stacked_bar_data = RwSignal::new(BarDataset::default());
    let _stacked_bar_res = LocalResource::new(move || async move {
        match fetch_csv(&format!("{BASE_PATH}/data/bar_stacked.csv")).await {
            Ok(table) => stacked_bar_data.set(to_grouped_bar(
                &table,
                "month",
                &[
                    ("desktop", "Desktop"),
                    ("mobile", "Mobile"),
                    ("tablet", "Tablet"),
                ],
            )),
            Err(e) => log::error!("bar_stacked.csv: {e}"),
        }
    });

    // Demo 8: Pie Chart
    let pie_data = RwSignal::new(vec![]);
    let _pie_res = LocalResource::new(move || async move {
        match fetch_csv(&format!("{BASE_PATH}/data/pie.csv")).await {
            Ok(table) => pie_data.set(to_pie_entries(&table, "label", "value")),
            Err(e) => log::error!("pie.csv: {e}"),
        }
    });

    // Demo 9: Parametric Chart — hardcoded (data is reactive to freq_param slider)
    let freq_param = RwSignal::new(1.0_f64);
    let parametric_data = Signal::derive(move || {
        let f = freq_param.get();
        let points: Vec<DataPoint> = (0..200)
            .map(|i| {
                let x = i as f64 * 0.1;
                DataPoint::new(x, (x * f).sin())
            })
            .collect();
        Dataset::from_series(Series::new(format!("sin({:.1} * x)", f), points))
    });

    // Demo 10: Box Plot
    let box_data = RwSignal::new(vec![]);
    let _box_res = LocalResource::new(move || async move {
        match fetch_csv(&format!("{BASE_PATH}/data/box_data.csv")).await {
            Ok(table) => box_data.set(to_box_groups(&table, "group", "value")),
            Err(e) => log::error!("box_data.csv: {e}"),
        }
    });

    // Demo 11: Histogram
    let hist_data = RwSignal::new(vec![]);
    let _hist_res = LocalResource::new(move || async move {
        match fetch_csv(&format!("{BASE_PATH}/data/histogram.csv")).await {
            Ok(table) => hist_data.set(to_histogram_values(&table, "value")),
            Err(e) => log::error!("histogram.csv: {e}"),
        }
    });

    // Demo 12: Candlestick
    let candle_data = RwSignal::new(vec![]);
    let _candle_res = LocalResource::new(move || async move {
        match fetch_csv(&format!("{BASE_PATH}/data/candlestick.csv")).await {
            Ok(table) => {
                candle_data.set(to_ohlc(&table, "bar", "open", "high", "low", "close"));
            }
            Err(e) => log::error!("candlestick.csv: {e}"),
        }
    });

    // Demo 13: Radar Chart
    let radar_axes = vec![
        "Performance".to_string(),
        "Reliability".to_string(),
        "Security".to_string(),
        "Scalability".to_string(),
        "UX".to_string(),
        "Cost".to_string(),
    ];
    let radar_data = RwSignal::new(vec![]);
    let _radar_res = LocalResource::new(move || async move {
        match fetch_csv(&format!("{BASE_PATH}/data/radar.csv")).await {
            Ok(table) => radar_data.set(to_radar_series(
                &table,
                "product",
                &[
                    "performance",
                    "reliability",
                    "security",
                    "scalability",
                    "ux",
                    "cost",
                ],
            )),
            Err(e) => log::error!("radar.csv: {e}"),
        }
    });

    // Demo 14: Waterfall
    let waterfall_data = RwSignal::new(vec![]);
    let _waterfall_res = LocalResource::new(move || async move {
        match fetch_csv(&format!("{BASE_PATH}/data/waterfall.csv")).await {
            Ok(table) => {
                waterfall_data.set(to_waterfall_bars(&table, "label", "value", "kind"));
            }
            Err(e) => log::error!("waterfall.csv: {e}"),
        }
    });

    // Demo 15: Violin Chart
    let violin_data = RwSignal::new(vec![]);
    let _violin_res = LocalResource::new(move || async move {
        match fetch_csv(&format!("{BASE_PATH}/data/violin_data.csv")).await {
            Ok(table) => violin_data.set(to_box_groups(&table, "group", "value")),
            Err(e) => log::error!("violin_data.csv: {e}"),
        }
    });

    // --- Chart configs (different grid styles) ---
    let config1 = RwSignal::new(
        ChartConfig::default()
            .with_title("Sine Wave (no grid)")
            .with_grid_visible(false),
    );
    let config2 = RwSignal::new(
        ChartConfig::default()
            .with_title("Large Dataset (dashed grid)")
            .with_grid(GridStyle {
                color: "#8888aa".to_string(),
                opacity: 0.4,
                width: 1.0,
                dash: Some("6,3".to_string()),
                show_x: true,
                show_y: true,
            }),
    );
    let config3 = RwSignal::new(
        ChartConfig::default()
            .with_title("Area Chart (Y grid only)")
            .with_grid(GridStyle {
                color: "#55aa55".to_string(),
                opacity: 0.25,
                width: 0.5,
                dash: None,
                show_x: false,
                show_y: true,
            }),
    );
    let config4 = RwSignal::new(
        ChartConfig::default()
            .with_title("Scatter Plot (dotted grid)")
            .with_grid(GridStyle {
                color: "#cc6666".to_string(),
                opacity: 0.35,
                width: 1.0,
                dash: Some("2,2".to_string()),
                show_x: true,
                show_y: true,
            }),
    );
    let config5 = RwSignal::new(
        ChartConfig::default()
            .with_title("Multi-Series (X grid only)")
            .with_grid(GridStyle {
                color: "#6688cc".to_string(),
                opacity: 0.3,
                width: 0.5,
                dash: None,
                show_x: true,
                show_y: false,
            }),
    );
    let config6 = RwSignal::new(
        ChartConfig::default()
            .with_title("Bar Chart (thick grid)")
            .with_grid(GridStyle {
                color: "#3366cc".to_string(),
                opacity: 0.15,
                width: 2.0,
                dash: None,
                show_x: true,
                show_y: true,
            }),
    );
    let config7 = RwSignal::new(ChartConfig::default().with_title("Stacked Bar (default grid)"));
    let config8 = RwSignal::new(ChartConfig::default().with_title("Browser Market Share"));
    let config9 = RwSignal::new(
        ChartConfig::default()
            .with_title("Parametric (dashed Y-only)")
            .with_grid(GridStyle {
                color: "#aa77cc".to_string(),
                opacity: 0.4,
                width: 1.0,
                dash: Some("4,4".to_string()),
                show_x: false,
                show_y: true,
            }),
    );
    let config10 = RwSignal::new(ChartConfig::default().with_title("Box Plot (Q1–Q4 Comparison)"));
    let config11 = RwSignal::new(ChartConfig::default().with_title("Histogram (FD binning)"));
    let config12 = RwSignal::new(ChartConfig::default().with_title("Candlestick (60 bars OHLC)"));
    let config13 =
        RwSignal::new(ChartConfig::default().with_title("Radar Chart (Product Comparison)"));
    let config14 = RwSignal::new(ChartConfig::default().with_title("Waterfall (P&L 2024)"));
    let config15 = RwSignal::new(ChartConfig::default().with_title("Violin Chart (Distributions)"));

    // Demo 16: Heatmap — inline 8×8 correlation matrix
    let heatmap_data = RwSignal::new({
        let n = 8usize;
        let labels: Vec<String> = (0..n).map(|i| format!("Var {}", i + 1)).collect();
        let values: Vec<Vec<f64>> = (0..n)
            .map(|i| {
                (0..n)
                    .map(|j| {
                        let d = (i as f64 - j as f64).abs() / n as f64;
                        1.0 - d
                    })
                    .collect()
            })
            .collect();
        GridData {
            values,
            row_labels: Some(labels.clone()),
            col_labels: Some(labels),
        }
    });

    // Demo 17: StripChart — 4 groups × 25 values
    let strip_data = RwSignal::new(vec![
        StripGroup {
            name: "Control".into(),
            values: (0..25)
                .map(|i| 2.0 + (i as f64 * 0.37_f64).sin() * 1.5)
                .collect(),
        },
        StripGroup {
            name: "Group A".into(),
            values: (0..25)
                .map(|i| 4.0 + (i as f64 * 0.53_f64).sin() * 2.0)
                .collect(),
        },
        StripGroup {
            name: "Group B".into(),
            values: (0..25)
                .map(|i| 3.5 + (i as f64 * 0.71_f64).cos() * 1.0)
                .collect(),
        },
        StripGroup {
            name: "Group C".into(),
            values: (0..25)
                .map(|i| 5.0 + (i as f64 * 0.43_f64).sin() * 2.5)
                .collect(),
        },
    ]);

    // Demo 18: SankeyChart — Energy flow
    let sankey_data = RwSignal::new(SankeyData {
        nodes: vec![
            SankeyNode {
                label: "Coal".into(),
                color: None,
            },
            SankeyNode {
                label: "Gas".into(),
                color: None,
            },
            SankeyNode {
                label: "Solar".into(),
                color: None,
            },
            SankeyNode {
                label: "Electricity".into(),
                color: None,
            },
            SankeyNode {
                label: "Heat".into(),
                color: None,
            },
            SankeyNode {
                label: "Industry".into(),
                color: None,
            },
            SankeyNode {
                label: "Transport".into(),
                color: None,
            },
            SankeyNode {
                label: "Buildings".into(),
                color: None,
            },
        ],
        links: vec![
            SankeyLink {
                source: 0,
                target: 3,
                value: 40.0,
                color: None,
            },
            SankeyLink {
                source: 0,
                target: 4,
                value: 20.0,
                color: None,
            },
            SankeyLink {
                source: 1,
                target: 3,
                value: 30.0,
                color: None,
            },
            SankeyLink {
                source: 1,
                target: 4,
                value: 25.0,
                color: None,
            },
            SankeyLink {
                source: 2,
                target: 3,
                value: 15.0,
                color: None,
            },
            SankeyLink {
                source: 3,
                target: 5,
                value: 35.0,
                color: None,
            },
            SankeyLink {
                source: 3,
                target: 6,
                value: 28.0,
                color: None,
            },
            SankeyLink {
                source: 3,
                target: 7,
                value: 22.0,
                color: None,
            },
            SankeyLink {
                source: 4,
                target: 5,
                value: 25.0,
                color: None,
            },
            SankeyLink {
                source: 4,
                target: 7,
                value: 20.0,
                color: None,
            },
        ],
    });

    // Demo 19: ChordChart — Trade flow between 5 regions
    let chord_data = RwSignal::new(ChordData {
        matrix: vec![
            vec![0.0, 12.0, 8.0, 5.0, 3.0],
            vec![7.0, 0.0, 15.0, 4.0, 6.0],
            vec![6.0, 11.0, 0.0, 9.0, 4.0],
            vec![3.0, 5.0, 8.0, 0.0, 12.0],
            vec![4.0, 7.0, 5.0, 10.0, 0.0],
        ],
        labels: vec!["Europe", "Americas", "Asia", "Africa", "Oceania"]
            .into_iter()
            .map(String::from)
            .collect(),
        colors: None,
    });

    // Demo 20: ContourChart — z = sin(x)*cos(y) on 40×40 grid
    let contour_data = RwSignal::new({
        let nx = 40usize;
        let ny = 40usize;
        let values = (0..ny)
            .map(|j| {
                (0..nx)
                    .map(|i| {
                        let x = (i as f64 / nx as f64) * std::f64::consts::TAU;
                        let y = (j as f64 / ny as f64) * std::f64::consts::TAU;
                        x.sin() * y.cos()
                    })
                    .collect()
            })
            .collect();
        GridData {
            values,
            row_labels: None,
            col_labels: None,
        }
    });

    let config16 = RwSignal::new(ChartConfig::default().with_title("Heatmap (Correlation Matrix)"));
    let config17 =
        RwSignal::new(ChartConfig::default().with_title("Strip Chart (Group Distributions)"));
    let config18 = RwSignal::new(ChartConfig::default().with_title("Sankey (Energy Flow)"));
    let config19 = RwSignal::new(ChartConfig::default().with_title("Chord Diagram (Trade Flow)"));
    let config20 = RwSignal::new(ChartConfig::default().with_title("Contour Chart (sin×cos)"));

    // Demo 21: DataTable — Sales performance dataset
    let table_data = RwSignal::new(TableData::default());
    let _table_res = LocalResource::new(move || async move {
        match crate::data::csv_loader::fetch_table_data(&format!(
            "{BASE_PATH}/data/sales_performance.csv"
        ))
        .await
        {
            Ok(mut table) => {
                // Fix column types for proper filtering
                for col in &mut table.columns {
                    match col.key.as_str() {
                        "MarginPct" | "Units" | "Revenue" => {
                            col.col_type = ColumnType::Number;
                            col.alignment = Alignment::Right;
                        }
                        "Target" => {
                            col.col_type = ColumnType::Boolean;
                        }
                        "Region" | "Status" | "Quarter" | "Product" => {
                            col.col_type = ColumnType::Category(vec![]);
                        }
                        _ => {}
                    }
                }
                table_data.set(table);
            }
            Err(e) => log::error!("sales_performance.csv: {e}"),
        }
    });

    let selected_chart = RwSignal::new(0);

    // Demo 22: SmartChart — Grammar of Graphics with reactive Mark switcher
    let smart_mark = RwSignal::new(Mark::Line);
    let smart_spec = Signal::derive(move || {
        let title = match smart_mark.get() {
            Mark::Line => "SmartChart — Mark::Line",
            Mark::Area => "SmartChart — Mark::Area",
            Mark::Point | Mark::Circle => "SmartChart — Mark::Point (Scatter)",
            _ => "SmartChart",
        };
        ChartSpec::builder()
            .data(sine_data.get())
            .mark(smart_mark.get())
            .x(Field::quantitative("x"))
            .title(title)
            .grid(true)
            .build()
    });

    view! {
        <ThemeProvider>
            <ErrorBoundary fallback=|errors| {
                view! {
                    <h1>"Uh oh! Something went wrong!"</h1>
                    <p>"Errors: "</p>
                    <ul>
                        {move || {
                            errors
                                .get()
                                .into_iter()
                                .map(|(_, e)| view! { <li>{e.to_string()}</li> })
                                .collect_view()
                        }}
                    </ul>
                }
            }>

                <div class="flex flex-col h-screen w-full overflow-hidden">
                    <Navbar />

                    <div class="flex flex-1 overflow-hidden">
                        // Sidebar
                        <div class="w-80 bg-base-200 border-r border-base-300 flex flex-col overflow-y-auto">
                            <div class="p-4 border-b border-base-300">
                                <h2 class="text-lg font-bold">"Theme Editor"</h2>
                                <p class="text-xs opacity-60">"Select a chart to edit"</p>
                            </div>

                            <div class="p-4 flex flex-col gap-4">
                                <div class="form-control">
                                    <label class="label">
                                        <span class="label-text">"Selected Chart"</span>
                                    </label>
                                    <select
                                        class="select select-bordered select-sm w-full"
                                        on:change=move |ev| {
                                            if let Ok(val) = event_target_value(&ev).parse::<usize>() {
                                                selected_chart.set(val);
                                            }
                                        }
                                        prop:value=move || selected_chart.get().to_string()
                                    >
                                        <option value="0">"1. Sine Wave"</option>
                                        <option value="1">"2. Large Dataset"</option>
                                        <option value="2">"3. Area Chart"</option>
                                        <option value="3">"4. Scatter Plot"</option>
                                        <option value="4">"5. Multi-Series"</option>
                                        <option value="5">"6. Bar Chart"</option>
                                        <option value="6">"7. Stacked Bar"</option>
                                        <option value="7">"8. Pie Chart"</option>
                                        <option value="8">"9. Parametric Demo"</option>
                                        <option value="9">"10. Box Plot"</option>
                                        <option value="10">"11. Histogram"</option>
                                        <option value="11">"12. Candlestick"</option>
                                        <option value="12">"13. Radar Chart"</option>
                                        <option value="13">"14. Waterfall"</option>
                                        <option value="14">"15. Violin Chart"</option>
                                        <option value="15">"16. Heatmap"</option>
                                        <option value="16">"17. Strip Chart"</option>
                                        <option value="17">"18. Sankey Chart"</option>
                                        <option value="18">"19. Chord Diagram"</option>
                                        <option value="19">"20. Contour Chart"</option>
                                    </select>
                                </div>

                                // Render editor for selected chart
                                {move || {
                                    let (name, sig) = match selected_chart.get() {
                                        0 => ("Sine Wave", config1),
                                        1 => ("Large Dataset", config2),
                                        2 => ("Area Chart", config3),
                                        3 => ("Scatter Plot", config4),
                                        4 => ("Multi-Series", config5),
                                        5 => ("Bar Chart", config6),
                                        6 => ("Stacked Bar", config7),
                                        7 => ("Pie Chart", config8),
                                        8 => ("Parametric Demo", config9),
                                        9 => ("Box Plot", config10),
                                        10 => ("Histogram", config11),
                                        11 => ("Candlestick", config12),
                                        12 => ("Radar Chart", config13),
                                        13 => ("Waterfall", config14),
                                        14 => ("Violin Chart", config15),
                                        15 => ("Heatmap", config16),
                                        16 => ("Strip Chart", config17),
                                        17 => ("Sankey Chart", config18),
                                        18 => ("Chord Diagram", config19),
                                        _ => ("Contour Chart", config20),
                                    };
                                    view! {
                                        <div class="text-xs font-mono opacity-50 mb-1">
                                            "Editing: " {name}
                                        </div>
                                        <ThemeEditor config=sig />

                                        // Parametric Controls (Only for Parametric Chart)
                                        {move || {
                                            if selected_chart.get() == 8 {
                                                view! {
                                                    <div class="divider my-2"></div>
                                                    <h3 class="text-sm font-bold mb-2">"Parameters"</h3>
                                                    <div class="form-control w-full">
                                                        <label class="label py-1">
                                                            <span class="label-text text-xs">"Frequency"</span>
                                                            <span class="label-text-alt text-xs">
                                                                {move || format!("{:.1}", freq_param.get())}
                                                            </span>
                                                        </label>
                                                        <input
                                                            type="range"
                                                            min="0.1"
                                                            max="5.0"
                                                            step="0.1"
                                                            class="range range-xs range-warning"
                                                            prop:value=move || freq_param.get()
                                                            on:input=move |ev| {
                                                                if let Ok(val) = event_target_value(&ev).parse::<f64>() {
                                                                    freq_param.set(val);
                                                                }
                                                            }
                                                        />
                                                    </div>
                                                }
                                                    .into_any()
                                            } else {
                                                ().into_any()
                                            }
                                        }}

                                        // Series Visibility Controls (Only for Multi-Series Chart)
                                        {move || {
                                            if selected_chart.get() == 4 {
                                                view! {
                                                    <div class="divider my-2"></div>
                                                    <h3 class="text-sm font-bold mb-2">"Series Visibility"</h3>
                                                    <div class="flex flex-col gap-2">
                                                        {move || {
                                                            multi_data
                                                                .get()
                                                                .series
                                                                .iter()
                                                                .enumerate()
                                                                .map(|(idx, series)| {
                                                                    let name = series.name.clone();
                                                                    let visible = series.visible;
                                                                    view! {
                                                                        <label class="label cursor-pointer py-1 justify-start gap-2">
                                                                            <input
                                                                                type="checkbox"
                                                                                class="checkbox checkbox-xs checkbox-primary"
                                                                                checked=visible
                                                                                on:change=move |ev| {
                                                                                    let checked = event_target_checked(&ev);
                                                                                    multi_data
                                                                                        .update(|d| {
                                                                                            if let Some(s) = d.series.get_mut(idx) {
                                                                                                s.visible = checked;
                                                                                            }
                                                                                        });
                                                                                }
                                                                            />
                                                                            <span class="label-text text-xs">{name}</span>
                                                                        </label>
                                                                    }
                                                                })
                                                                .collect_view()
                                                        }}
                                                    </div>
                                                }
                                                    .into_any()
                                            } else {
                                                ().into_any()
                                            }
                                        }}
                                    }
                                }}
                            </div>
                        </div>

                        // Main Content
                        <div class="flex-1 overflow-y-auto p-8 relative">

                            // Dashboard container
                            <div class="dashboard relative min-h-[6100px] bg-base-200 rounded-box p-4">
                                // Linked dashboard: rows 1–3 share crosshair on hover
                                <LinkedDashboard>

                                // Row 1: Sine Wave + Large Dataset
                                <DraggableCard
                                    initial_transform=CardTransform {
                                        x: 20.0,
                                        y: 20.0,
                                        width: 860.0,
                                        height: 400.0,
                                    }
                                    color_scheme=ColorScheme::Ocean
                                    snap_size=10
                                    min_width=400.0
                                    min_height=300.0
                                >
                                    <ChartVisibility>
                                        <LineChart
                                            data=sine_data.into()
                                            config=config1
                                            x_label="Index".to_string()
                                            y_label="Amplitude".to_string()
                                        />
                                    </ChartVisibility>
                                    <div style="position: absolute; bottom: 6px; right: 8px; z-index: 5; font-size: 9px; opacity: 0.45; pointer-events: none; background: rgba(0,0,0,0.35); color: white; padding: 2px 6px; border-radius: 4px; font-family: monospace;">
                                        "ℹ Ctrl+drag zoom · dbl-click reset"
                                    </div>
                                </DraggableCard>

                                <DraggableCard
                                    initial_transform=CardTransform {
                                        x: 900.0,
                                        y: 20.0,
                                        width: 860.0,
                                        height: 400.0,
                                    }
                                    color_scheme=ColorScheme::Sunset
                                    snap_size=10
                                    min_width=400.0
                                    min_height=300.0
                                >
                                    <ChartVisibility>
                                        <LineChart
                                            data=large_data.into()
                                            config=config2
                                            show_grid=true
                                        />
                                    </ChartVisibility>
                                    <div style="position: absolute; bottom: 6px; right: 8px; z-index: 5; font-size: 9px; opacity: 0.45; pointer-events: none; background: rgba(0,0,0,0.35); color: white; padding: 2px 6px; border-radius: 4px; font-family: monospace;">
                                        "ℹ Ctrl+drag zoom · dbl-click reset"
                                    </div>
                                </DraggableCard>

                                // Row 2: Area Chart + Scatter Plot
                                <DraggableCard
                                    initial_transform=CardTransform {
                                        x: 20.0,
                                        y: 440.0,
                                        width: 860.0,
                                        height: 400.0,
                                    }
                                    color_scheme=ColorScheme::Viridis
                                    snap_size=10
                                    min_width=400.0
                                    min_height=300.0
                                >
                                    <ChartVisibility>
                                        <AreaChart
                                            data=area_data.into()
                                            config=config3
                                            show_grid=true
                                            x_label="Month".to_string()
                                            y_label="Amount ($K)".to_string()
                                        />
                                    </ChartVisibility>
                                    <div style="position: absolute; bottom: 6px; right: 8px; z-index: 5; font-size: 9px; opacity: 0.45; pointer-events: none; background: rgba(0,0,0,0.35); color: white; padding: 2px 6px; border-radius: 4px; font-family: monospace;">
                                        "ℹ Ctrl+drag zoom · dbl-click reset"
                                    </div>
                                </DraggableCard>

                                <DraggableCard
                                    initial_transform=CardTransform {
                                        x: 900.0,
                                        y: 440.0,
                                        width: 860.0,
                                        height: 400.0,
                                    }
                                    color_scheme=ColorScheme::DarkMatter
                                    snap_size=10
                                    min_width=400.0
                                    min_height=300.0
                                >
                                    <ChartVisibility>
                                        <ScatterChart
                                            data=scatter_data.into()
                                            config=config4
                                            show_grid=true
                                            x_label="X".to_string()
                                            y_label="Y".to_string()
                                        />
                                    </ChartVisibility>
                                </DraggableCard>

                                // Row 3: Multi-Series (full width)
                                <DraggableCard
                                    initial_transform=CardTransform {
                                        x: 20.0,
                                        y: 860.0,
                                        width: 1740.0,
                                        height: 420.0,
                                    }
                                    color_scheme=ColorScheme::Categorical
                                    snap_size=10
                                    min_width=600.0
                                    min_height=350.0
                                >
                                    <ChartVisibility>
                                        <LineChart
                                            data=multi_data.into()
                                            config=config5
                                            show_grid=true
                                            x_label="Time".to_string()
                                            y_label="Value".to_string()
                                        />
                                    </ChartVisibility>
                                </DraggableCard>

                                </LinkedDashboard>

                                // Row 4: Bar Charts + Pie Chart
                                <DraggableCard
                                    initial_transform=CardTransform {
                                        x: 20.0,
                                        y: 1300.0,
                                        width: 580.0,
                                        height: 400.0,
                                    }
                                    color_scheme=ColorScheme::Ocean
                                    snap_size=10
                                    min_width=350.0
                                    min_height=300.0
                                >
                                    <ChartVisibility>
                                        <BarChart
                                            data=bar_data.into()
                                            config=config6
                                            show_grid=true
                                            y_label="Sales ($K)".to_string()
                                        />
                                    </ChartVisibility>
                                </DraggableCard>

                                <DraggableCard
                                    initial_transform=CardTransform {
                                        x: 620.0,
                                        y: 1300.0,
                                        width: 580.0,
                                        height: 400.0,
                                    }
                                    color_scheme=ColorScheme::Sunset
                                    snap_size=10
                                    min_width=350.0
                                    min_height=300.0
                                >
                                    <ChartVisibility>
                                        <BarChart
                                            data=stacked_bar_data.into()
                                            config=config7
                                            show_grid=true
                                            mode=BarMode::Stacked
                                            y_label="Visits".to_string()
                                        />
                                    </ChartVisibility>
                                </DraggableCard>

                                <DraggableCard
                                    initial_transform=CardTransform {
                                        x: 1220.0,
                                        y: 1300.0,
                                        width: 540.0,
                                        height: 400.0,
                                    }
                                    color_scheme=ColorScheme::Viridis
                                    snap_size=10
                                    min_width=300.0
                                    min_height=300.0
                                >
                                    <ChartVisibility>
                                        <PieChart data=pie_data.into() config=config8 donut=true />
                                    </ChartVisibility>
                                </DraggableCard>

                                // Row 5: Parametric Demo (hardcoded — reactive to slider)
                                <DraggableCard
                                    initial_transform=CardTransform {
                                        x: 20.0,
                                        y: 1720.0,
                                        width: 860.0,
                                        height: 400.0,
                                    }
                                    color_scheme=ColorScheme::DarkMatter
                                    snap_size=10
                                    min_width=400.0
                                    min_height=300.0
                                >
                                    <ChartVisibility>
                                        <LineChart
                                            data=parametric_data
                                            config=config9
                                            show_grid=true
                                            x_label="Time".to_string()
                                            y_label="Amplitude".to_string()
                                        />
                                    </ChartVisibility>
                                </DraggableCard>

                                // Row 6: Box Plot + Violin Chart
                                <DraggableCard
                                    initial_transform=CardTransform {
                                        x: 20.0,
                                        y: 2150.0,
                                        width: 860.0,
                                        height: 400.0,
                                    }
                                    color_scheme=ColorScheme::Categorical
                                    snap_size=10
                                    min_width=400.0
                                    min_height=300.0
                                >
                                    <ChartVisibility>
                                        <BoxPlot
                                            data=box_data.into()
                                            config=config10
                                            y_label="Value".to_string()
                                        />
                                    </ChartVisibility>
                                </DraggableCard>

                                <DraggableCard
                                    initial_transform=CardTransform {
                                        x: 900.0,
                                        y: 2150.0,
                                        width: 860.0,
                                        height: 400.0,
                                    }
                                    color_scheme=ColorScheme::Ocean
                                    snap_size=10
                                    min_width=400.0
                                    min_height=300.0
                                >
                                    <ChartVisibility>
                                        <ViolinChart
                                            data=violin_data.into()
                                            config=config15
                                            y_label="Value".to_string()
                                        />
                                    </ChartVisibility>
                                </DraggableCard>

                                // Row 7: Histogram + Candlestick
                                <DraggableCard
                                    initial_transform=CardTransform {
                                        x: 20.0,
                                        y: 2570.0,
                                        width: 580.0,
                                        height: 400.0,
                                    }
                                    color_scheme=ColorScheme::Viridis
                                    snap_size=10
                                    min_width=350.0
                                    min_height=300.0
                                >
                                    <ChartVisibility>
                                        <Histogram
                                            data=hist_data.into()
                                            config=config11
                                            x_label="Value".to_string()
                                            y_label="Count".to_string()
                                        />
                                    </ChartVisibility>
                                </DraggableCard>

                                <DraggableCard
                                    initial_transform=CardTransform {
                                        x: 620.0,
                                        y: 2570.0,
                                        width: 1160.0,
                                        height: 400.0,
                                    }
                                    color_scheme=ColorScheme::DarkMatter
                                    snap_size=10
                                    min_width=400.0
                                    min_height=300.0
                                >
                                    <ChartVisibility>
                                        <CandlestickChart
                                            data=candle_data.into()
                                            config=config12
                                            x_label="Bar".to_string()
                                            y_label="Price".to_string()
                                        />
                                    </ChartVisibility>
                                </DraggableCard>

                                // Row 8: Radar Chart + Waterfall
                                <DraggableCard
                                    initial_transform=CardTransform {
                                        x: 20.0,
                                        y: 2990.0,
                                        width: 580.0,
                                        height: 400.0,
                                    }
                                    color_scheme=ColorScheme::Sunset
                                    snap_size=10
                                    min_width=350.0
                                    min_height=300.0
                                >
                                    <ChartVisibility>
                                        <RadarChart
                                            axes=radar_axes.clone()
                                            data=radar_data.into()
                                            config=config13
                                            max_value=100.0
                                        />
                                    </ChartVisibility>
                                </DraggableCard>

                                <DraggableCard
                                    initial_transform=CardTransform {
                                        x: 620.0,
                                        y: 2990.0,
                                        width: 1160.0,
                                        height: 400.0,
                                    }
                                    color_scheme=ColorScheme::Categorical
                                    snap_size=10
                                    min_width=400.0
                                    min_height=300.0
                                >
                                    <ChartVisibility>
                                        <WaterfallChart
                                            data=waterfall_data.into()
                                            config=config14
                                            y_label="Value ($K)".to_string()
                                        />
                                    </ChartVisibility>
                                </DraggableCard>

                                // Row 6: Heatmap + Strip Chart
                                <DraggableCard
                                    initial_transform=CardTransform {
                                        x: 20.0,
                                        y: 3420.0,
                                        width: 860.0,
                                        height: 500.0,
                                    }
                                    color_scheme=ColorScheme::Viridis
                                    snap_size=10
                                    min_width=400.0
                                    min_height=350.0
                                >
                                    <ChartVisibility>
                                        <HeatmapChart
                                            data=heatmap_data.into()
                                            config=config16
                                            show_colorbar=true
                                            show_values=false
                                            color_map=ColorMap::Sequential(SequentialColorMap::Viridis)
                                        />
                                    </ChartVisibility>
                                </DraggableCard>

                                <DraggableCard
                                    initial_transform=CardTransform {
                                        x: 900.0,
                                        y: 3420.0,
                                        width: 860.0,
                                        height: 500.0,
                                    }
                                    color_scheme=ColorScheme::Ocean
                                    snap_size=10
                                    min_width=400.0
                                    min_height=350.0
                                >
                                    <ChartVisibility>
                                        <StripChart
                                            data=strip_data.into()
                                            layout=StripLayout::Jitter
                                            config=config17
                                        />
                                    </ChartVisibility>
                                </DraggableCard>

                                // Row 7: Sankey Chart
                                <DraggableCard
                                    initial_transform=CardTransform {
                                        x: 20.0,
                                        y: 3950.0,
                                        width: 1740.0,
                                        height: 500.0,
                                    }
                                    color_scheme=ColorScheme::Sunset
                                    snap_size=10
                                    min_width=500.0
                                    min_height=350.0
                                >
                                    <ChartVisibility>
                                        <SankeyChart data=sankey_data.into() config=config18 />
                                    </ChartVisibility>
                                </DraggableCard>

                                // Row 8: Chord Diagram + Contour Chart
                                <DraggableCard
                                    initial_transform=CardTransform {
                                        x: 20.0,
                                        y: 4480.0,
                                        width: 860.0,
                                        height: 500.0,
                                    }
                                    color_scheme=ColorScheme::Categorical
                                    snap_size=10
                                    min_width=400.0
                                    min_height=350.0
                                >
                                    <ChartVisibility>
                                        <ChordChart data=chord_data.into() config=config19 />
                                    </ChartVisibility>
                                </DraggableCard>

                                <DraggableCard
                                    initial_transform=CardTransform {
                                        x: 900.0,
                                        y: 4480.0,
                                        width: 860.0,
                                        height: 500.0,
                                    }
                                    color_scheme=ColorScheme::Viridis
                                    snap_size=10
                                    min_width=400.0
                                    min_height=350.0
                                >
                                    <ChartVisibility>
                                        <ContourChart
                                            data=contour_data.into()
                                            config=config20
                                            levels=10
                                            filled=true
                                            color_map=ColorMap::Sequential(SequentialColorMap::Plasma)
                                        />
                                    </ChartVisibility>
                                </DraggableCard>

                                // ── Demo 21: DataTable ──────────────────────────────────
                                <DraggableCard
                                    initial_transform=CardTransform {
                                        x: 20.0,
                                        y: 5010.0,
                                        width: 1160.0,
                                        height: 520.0,
                                    }
                                    // Added color scheme
                                    color_scheme=ColorScheme::Ocean
                                    snap_size=10
                                    min_width=600.0
                                    min_height=400.0
                                >
                                    <ChartVisibility>
                                        <DataTable
                                            data=table_data.into()
                                            title="Sales Performance — Prodotti × Regioni"
                                            default_page_size=10usize
                                        />
                                    </ChartVisibility>
                                </DraggableCard>

                                // ── Demo 22: SmartChart — Grammar of Graphics ──────────
                                <DraggableCard
                                    initial_transform=CardTransform {
                                        x: 20.0,
                                        y: 5560.0,
                                        width: 860.0,
                                        height: 420.0,
                                    }
                                    color_scheme=ColorScheme::Categorical
                                    snap_size=10
                                    min_width=400.0
                                    min_height=300.0
                                >
                                    <ChartVisibility>
                                        <div style="display: flex; flex-direction: column; width: 100%; height: 100%;">
                                            <div style="padding: 4px 12px 0; display: flex; align-items: center; gap: 8px; flex-shrink: 0;">
                                                <span style="font-size: 11px; opacity: 0.65; font-weight: 600;">"Grammar of Graphics — Mark:"</span>
                                                <select
                                                    class="select select-bordered select-xs"
                                                    on:change=move |ev| {
                                                        let mark = match event_target_value(&ev).as_str() {
                                                            "area" => Mark::Area,
                                                            "point" => Mark::Point,
                                                            _ => Mark::Line,
                                                        };
                                                        smart_mark.set(mark);
                                                    }
                                                >
                                                    <option value="line">"Line"</option>
                                                    <option value="area">"Area"</option>
                                                    <option value="point">"Point (Scatter)"</option>
                                                </select>
                                            </div>
                                            <div style="flex: 1; min-height: 0;">
                                                <SmartChart spec=smart_spec />
                                            </div>
                                        </div>
                                    </ChartVisibility>
                                </DraggableCard>
                            </div>
                            <footer class="text-center mt-8 text-base-content/40 text-sm">
                                <p>"Interactive Dashboard with 22 Chart Types + DataTable"</p>
                                <p>
                                    "Built with Leptos 0.8 + Trunk + WASM + Tailwind 4 + DaisyUI 5"
                                </p>
                            </footer>
                        </div>
                    </div>
                </div>
            </ErrorBoundary>
        </ThemeProvider>
    }
}

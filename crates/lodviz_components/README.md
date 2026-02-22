# lodviz_components

[![Crates.io](https://img.shields.io/crates/v/lodviz_components.svg)](https://crates.io/crates/lodviz_components)
[![Docs.rs](https://docs.rs/lodviz_components/badge.svg)](https://docs.rs/lodviz_components)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

High-level [Leptos](https://leptos.dev) components for data visualization, built on top of [`lodviz_core`](../lodviz_core).
Renders pure SVG — no JavaScript charting library required.

> **Target**: `wasm32-unknown-unknown` (Leptos 0.8 CSR). Requires [Trunk](https://trunkrs.dev) to build.

## Chart Components

| Component | Description |
|-----------|-------------|
| `LineChart` | Multi-series line chart with LTTB auto-downsampling |
| `BarChart` | Grouped and stacked bar chart |
| `ScatterChart` | X/Y scatter plot with configurable point shapes |
| `AreaChart` | Filled area chart, supports stacking |
| `BoxPlot` / `ViolinChart` | Statistical distribution charts |
| `Histogram` | Frequency distribution with configurable bins |
| `PieChart` | Pie and donut charts |
| `RadarChart` | Multi-axis radar / spider chart |
| `CandlestickChart` | OHLC financial chart with M4 downsampling |
| `WaterfallChart` | Running total waterfall |
| `SmartChart` | Facade that selects the renderer from a `ChartSpec` |

## Interactive Features

- **Zoom & Pan** — `ZoomPan` wrapper with mouse/touch support
- **Brush selection** — `Brush` for range selection across linked charts
- **Linked dashboards** — `LinkedDashboard` + `DashboardContext` for synchronized crosshair/selection
- **Draggable cards** — `DraggableCard` layout component for resizable dashboard panels
- **Tooltips** — Per-chart tooltip overlays with hover state
- **Legend toggle** — Click legend entries to show/hide series

### Zoom Interactions

Charts wrapped with `ZoomPan` support the following mouse interactions:

| Interaction | Behavior |
|-------------|----------|
| `Ctrl + drag` | Box zoom to selected area |
| `Double click` | Reset to original zoom level |
| `Mouse wheel` | Zoom in/out at cursor position |

```rust,ignore
use lodviz_components::{LineChart, ZoomPan};
use leptos::prelude::*;

let (transform, set_transform) = signal(ZoomTransform::from_domain(0.0, 100.0, 0.0, 50.0));
let original = transform.read_only();

view! {
    <ZoomPan
        transform=set_transform
        original=original
        inner_width=Signal::derive(|| 800.0)
        inner_height=Signal::derive(|| 400.0)
    >
        <LineChart data=data transform=transform />
    </ZoomPan>
}
```

## Installation

```toml
[dependencies]
lodviz_components = "0.1"
lodviz_core = "0.1"
```

Configure your `Trunk.toml` to target WASM:

```toml
[build]
target = "index.html"
```

## Quick Start

```rust,ignore
use lodviz_components::LineChart;
use lodviz_core::core::data::{DataPoint, Dataset, Series};
use leptos::prelude::*;

#[component]
fn App() -> impl IntoView {
    let data = Signal::derive(|| Dataset {
        series: vec![Series {
            label: "Sales".to_string(),
            points: (0..50)
                .map(|i| DataPoint::new(i as f64, (i as f64).sin() * 100.0))
                .collect(),
            color: None,
        }],
    });

    view! {
        <LineChart data=data />
    }
}
```

## Data Input

Components accept data **exclusively as `Signal<T>`**, where `T` is one of the chart-ready
types defined in `lodviz_core::core::data`. No parsing happens inside components.

| Component | Signal type |
|-----------|-------------|
| `LineChart`, `AreaChart`, `ScatterChart` | `Signal<Dataset>` |
| `BarChart` | `Signal<BarDataset>` |
| `CandlestickChart` | `Signal<Vec<OhlcBar>>` |
| `WaterfallChart` | `Signal<Vec<WaterfallBar>>` |
| `Histogram`, `BoxPlot`, `ViolinChart` | `Signal<Vec<f64>>` |
| `SmartChart` | `Signal<ChartSpec>` (generic facade) |

### From static data

```rust,ignore
use lodviz_core::core::data::{DataPoint, Dataset, Series};
use lodviz_components::LineChart;
use leptos::prelude::*;

let data = Signal::derive(|| Dataset::from_series(
    Series::new("Temp", vec![
        DataPoint::new(0.0, 22.5),
        DataPoint::new(1.0, 23.1),
    ])
));

view! { <LineChart data=data /> }
```

### From a remote CSV (browser fetch)

This crate performs no I/O. Data loading is the application's responsibility:

```rust,ignore
use lodviz_core::core::csv::parse_csv;
use lodviz_core::core::encoding::{Encoding, Field};
use leptos::prelude::*;

// LocalResource performs the fetch in WASM
let dataset = LocalResource::new(|| async {
    let text = gloo_net::http::Request::get("/data/sales.csv")
        .send().await.unwrap()
        .text().await.unwrap();

    let table = parse_csv(&text).unwrap();
    let enc = Encoding::new(
        Field::quantitative("month"),
        Field::quantitative("value"),
    );
    table.to_dataset(&enc)
});
```

For the full pipeline (CSV → DataTable → Dataset → downsampling)
see the **Data Pipeline** section in the [`lodviz_core` README](../lodviz_core).

## Theming

Wrap your app in `ThemeProvider` for automatic light/dark mode support:

```rust,ignore
use lodviz_components::ThemeProvider;

view! {
    <ThemeProvider>
        <LineChart data=data />
    </ThemeProvider>
}
```

Charts respond to the `prefers-color-scheme` media query automatically.

## Demo App

See [`apps/web_dashboard`](../../apps/web_dashboard) for a full working demo with routing,
multiple chart types, theme switching, and interactive dashboards.

```sh
cd apps/web_dashboard
trunk serve --port 3000
```

## License

MIT — see [LICENSE](../../LICENSE)

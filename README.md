# lodviz-rs

[![lodviz_core on crates.io](https://img.shields.io/crates/v/lodviz_core.svg?label=lodviz_core)](https://crates.io/crates/lodviz_core)
[![lodviz_components on crates.io](https://img.shields.io/crates/v/lodviz_components.svg?label=lodviz_components)](https://crates.io/crates/lodviz_components)
[![docs.rs lodviz_core](https://docs.rs/lodviz_core/badge.svg)](https://docs.rs/lodviz_core)
[![docs.rs lodviz_components](https://docs.rs/lodviz_components/badge.svg)](https://docs.rs/lodviz_components)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A pure-Rust, SVG-based data visualization library — a self-contained alternative to ECharts,
built on [Leptos](https://leptos.dev) 0.8 and compiled to WebAssembly.

**[Live Demo →](https://lodviz-rs.github.io/lodviz-rs)**

---

## What's in this repo

| Crate / App | Description | Target |
|-------------|-------------|--------|
| [`crates/lodviz_core`](crates/lodviz_core) | Core primitives: data model, scales, encoding, algorithms (LTTB, M4, KDE…) | native + WASM |
| [`crates/lodviz_components`](crates/lodviz_components) | Chart components built on `lodviz_core` | `wasm32-unknown-unknown` |
| [`apps/web_dashboard`](apps/web_dashboard) | Interactive demo app — hosted on GitHub Pages | WASM |

---

## Chart Components

| Component | Description |
|-----------|-------------|
| `LineChart` | Multi-series line chart with automatic LTTB downsampling (>1 000 pts) |
| `BarChart` | Grouped and stacked bar chart |
| `ScatterChart` | X/Y scatter plot |
| `AreaChart` | Filled area chart, supports stacking |
| `BoxPlot` / `ViolinChart` | Statistical distribution charts |
| `Histogram` | Frequency distribution with configurable bins |
| `PieChart` | Pie and donut charts |
| `RadarChart` | Multi-axis radar / spider chart |
| `CandlestickChart` | OHLC financial chart with M4 downsampling |
| `WaterfallChart` | Running total waterfall |
| `SmartChart` | Facade: picks the right renderer from a declarative `ChartSpec` |

All charts render **pure inline SVG** — no JavaScript charting library, no Canvas.

---

## Interactive Features

- **Zoom & Pan** — mouse/touch via `ZoomPan`
- **Brush selection** — range selection with `Brush`
- **Linked dashboards** — synchronized crosshair/highlight across charts with `LinkedDashboard`
- **Draggable cards** — resizable dashboard panels with `DraggableCard`
- **Tooltips** — per-chart hover overlays
- **Legend toggle** — click to show/hide individual series
- **Dark / Light theme** — automatic via `ThemeProvider` (`prefers-color-scheme`)

---

## Using the crates in your project

### `lodviz_core` — pure data logic (no UI dependency)

```toml
[dependencies]
lodviz_core = "0.1"
```

```rust
use lodviz_core::core::data::{DataPoint, Dataset, Series};
use lodviz_core::algorithms::lttb::lttb_downsample;
use lodviz_core::core::scale::LinearScale;

// Build a dataset
let series = Series::new(
    "temperature",
    (0..10_000)
        .map(|i| DataPoint::new(i as f64, (i as f64 * 0.01).sin() * 20.0))
        .collect(),
);
let dataset = Dataset::from_series(series);

// Downsample 10 000 → 300 points while preserving visual shape
let reduced = lttb_downsample(&dataset.series[0].data, 300);

// Map data values to pixel positions
let x_scale = LinearScale::from_extent(0.0, 10_000.0, 0.0, 800.0);
let px = x_scale.map(5_000.0); // → 400.0
```

### `lodviz_components` — Chart components

Requires a WASM target and [Trunk](https://trunkrs.dev).

```toml
[dependencies]
lodviz_components = "0.1"
lodviz_core = "0.1"
```

```rust,ignore
use lodviz_components::components::charts::line_chart::LineChart;
use lodviz_core::core::data::{DataPoint, Dataset, Series};
use leptos::prelude::*;

#[component]
fn App() -> impl IntoView {
    let data = Signal::derive(|| Dataset::from_series(Series::new(
        "sin",
        (0..200).map(|i| DataPoint::new(i as f64, (i as f64 * 0.05).sin())).collect(),
    )));

    view! { <LineChart data=data /> }
}
```

---

## Running the Demo Locally

### Prerequisites

```sh
# Rust nightly + WASM target
rustup toolchain install nightly
rustup target add wasm32-unknown-unknown --toolchain nightly

# Trunk (WASM bundler)
cargo install trunk
```

### Start the dev server

```sh
cd apps/web_dashboard
trunk serve --port 3000
```

Open <http://localhost:3000> — hot-reload is enabled.

### Production build

```sh
cd apps/web_dashboard
trunk build --release
```

Output goes to `apps/web_dashboard/dist/`. Deploy the contents of `dist/` to any static host
(GitHub Pages, Netlify, Cloudflare Pages, …).

---

## Workspace Structure

```
lodviz-rs/
├── crates/
│   ├── lodviz_core/          # Pure logic — no UI deps
│   │   ├── src/
│   │   │   ├── core/      # data, encoding, scale, mark, theme, a11y, spec
│   │   │   └── algorithms/ # lttb, m4, statistics, stack, nearest, arc
│   │   └── examples/      # Runnable: lttb.rs, scales.rs, basic_data.rs
│   └── lodviz_components/    # Chart components
│       └── src/components/
│           ├── charts/    # LineChart, BarChart, ScatterChart, …
│           ├── svg/       # Axis, Grid, Tooltip, Legend, Overlay
│           ├── interaction/ # ZoomPan, Brush, LinkedDashboard
│           └── layout/    # DraggableCard, ChartVisibility, GlobalMouse
├── apps/
│   └── web_dashboard/     # Demo app (→ GitHub Pages)
│       └── public/data/   # Static CSV files loaded at runtime
├── CHANGELOG.md
└── pub-crate.md           # Publishing guide for crates.io
```

---

## Key Design Decisions

- **CSR only** — no SSR, no `cargo-leptos`, no server functions. Trunk compiles directly to WASM.
- **Pure SVG** — all rendering goes through Leptos `view!{}` macros. No Canvas/WebGL.
- **LOD via LTTB** — series with >1 000 points are automatically downsampled before rendering,
  preserving visual fidelity ([Steinarsson 2013](http://skemman.is/stream/get/1946/15343/37285/3/SS_MSthesis.pdf)).
- **Grammar of Graphics** — declarative `Encoding` + `Field` + `Scale` API inspired by Vega-Lite.
- **Fine-grained reactivity** — Leptos 0.8 signals, no virtual DOM diffing.

---

## Development

```sh
# Lint (WASM target — catches wasm32-incompatible APIs)
cargo clippy --target wasm32-unknown-unknown -p lodviz_core -p lodviz_components

# Unit tests (native target — algorithms, scales, data structures)
cargo test -p lodviz_core

# Run a specific example
cargo run --example lttb -p lodviz_core
cargo run --example scales -p lodviz_core

# Format
cargo fmt
```

---

## License

MIT — see [LICENSE](LICENSE).

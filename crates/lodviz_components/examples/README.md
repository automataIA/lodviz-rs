# lodviz_components — Examples

`lodviz_components` targets `wasm32-unknown-unknown` and requires a browser to run.
Runnable examples are provided as pages in the demo app:

## Demo App

```sh
cd apps/web_dashboard
trunk serve --port 3000
```

Then open <http://localhost:3000> in your browser.

## Available Pages

| Route | What it shows |
|-------|---------------|
| `/` | Overview — all chart types in a grid |
| `/line` | `LineChart` with multi-series and LTTB downsampling |
| `/bar` | `BarChart` grouped and stacked modes |
| `/scatter` | `ScatterChart` with brush selection |
| `/area` | `AreaChart` stacked |
| `/candlestick` | `CandlestickChart` with M4 downsampling |
| `/dashboard` | `LinkedDashboard` with zoom/pan and brushing |
| `/theme` | `ThemeProvider` light/dark mode toggle |

## Writing your own chart

Add `lodviz_components` to your Leptos app's `Cargo.toml`, create an `index.html`,
configure `Trunk.toml`, then:

```rust,ignore
use lodviz_components::LineChart;
use lodviz_core::core::data::{DataPoint, Dataset, Series};
use leptos::prelude::*;

#[component]
fn App() -> impl IntoView {
    let data = Signal::derive(|| Dataset::from_series(Series::new(
        "demo",
        (0..100).map(|i| DataPoint::new(i as f64, (i as f64 * 0.1).sin())).collect(),
    )));

    view! { <LineChart data=data /> }
}
```

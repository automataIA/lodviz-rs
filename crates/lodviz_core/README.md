# lodviz_core

[![Crates.io](https://img.shields.io/crates/v/lodviz_core.svg)](https://crates.io/crates/lodviz_core)
[![Docs.rs](https://docs.rs/lodviz_core/badge.svg)](https://docs.rs/lodviz_core)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Core visualization primitives and data structures for [`lodviz-rs`](https://github.com/lodviz-rs/lodviz-rs) — a pure-Rust, SVG-based data visualization library.

## Features

- **Tidy Data Model** — `DataTable`, `DataRow`, and `FieldValue` for heterogeneous, column-oriented data
- **Grammar of Graphics** — Declarative `Encoding` and `Field` types inspired by Vega-Lite
- **Scales** — `LinearScale`, `BandScale`, and `OrdinalScale` for mapping data domains to screen ranges
- **LTTB Downsampling** — Largest-Triangle-Three-Buckets algorithm for visual-preserving time-series reduction
- **M4 Downsampling** — Fast Min-Max-Min-Max algorithm for large OHLC/financial datasets
- **Statistical Algorithms** — KDE, box-plot stats, mean, median, percentiles
- **Theming** — `ChartConfig` and palette definitions reused across renderers
- **Accessibility** — A11y primitives for screen-reader friendly SVG output
- **No `std` / WASM-compatible** — Pure logic, no OS runtime dependencies

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
lodviz_core = "0.1"
```

## Usage

### Working with data

```rust
use lodviz_core::core::data::{DataPoint, Series, Dataset};

let series = Series {
    label: "Temperature".to_string(),
    points: (0..100)
        .map(|i| DataPoint::new(i as f64, (i as f64 * 0.1).sin() * 20.0 + 15.0))
        .collect(),
    color: None,
};

let dataset = Dataset { series: vec![series] };
```

### LTTB downsampling

Reduce 10 000 points to 300 while preserving the visual shape:

```rust
use lodviz_core::core::data::DataPoint;
use lodviz_core::algorithms::lttb::lttb_downsample;

let data: Vec<DataPoint> = (0..10_000)
    .map(|i| DataPoint::new(i as f64, (i as f64 * 0.01).sin()))
    .collect();

let reduced = lttb_downsample(&data, 300);
assert_eq!(reduced.len(), 300);
```

### Scales

```rust
use lodviz_core::core::scale::LinearScale;

let scale = LinearScale::new(0.0, 100.0, 0.0, 600.0);
let px = scale.map(50.0); // → 300.0
```

### Encoding specification

```rust
use lodviz_core::core::encoding::{Encoding, Field};

let enc = Encoding::new()
    .x(Field::quantitative("timestamp"))
    .y(Field::quantitative("value"))
    .color(Field::nominal("series"));
```

## Data Pipeline

`lodviz_core` accetta dati a **tre livelli di astrazione**, dal più basso al più alto:

```
[1] Vec<DataPoint>          ← raw Rust types, nessun parsing
[2] DataTable / DataRow     ← tidy model, colonne eterogenee
[3] parse_csv(&str)         ← CSV testuale → DataTable
```

### Livello 1 — Tipi Rust puri

Il tipo di ingresso nativo per i chart è `Vec<DataPoint>` (x/y di tipo `f64`),
raggruppato in `Series<DataPoint>` e poi in `Dataset`.
Nessun parsing, nessuna allocazione extra: costruisci i dati direttamente.

```rust
use lodviz_core::core::data::{DataPoint, Dataset, Series};

let series = Series::new(
    "Revenue",
    (0..12).map(|i| DataPoint::new(i as f64, revenue[i])).collect(),
);
let dataset = Dataset::from_series(series);
```

Per chart specializzati esistono tipi dedicati:

| Tipo | Usato da |
|------|----------|
| `Dataset` (`Vec<Series<DataPoint>>`) | `LineChart`, `ScatterChart`, `AreaChart` |
| `BarDataset` | `BarChart` |
| `Vec<OhlcBar>` | `CandlestickChart` |
| `Vec<WaterfallBar>` | `WaterfallChart` |

### Livello 2 — Tidy Data Model

Se i tuoi dati hanno colonne eterogenee (mix di numeri, testo, timestamp),
usa `DataTable`. Ogni riga è una `DataRow` (`HashMap<String, FieldValue>`).

```rust
use lodviz_core::core::field_value::{DataTable, DataRow, FieldValue};
use lodviz_core::core::encoding::{Encoding, Field};

// Costruzione manuale
let mut row: DataRow = DataRow::new();
row.insert("month".into(), FieldValue::Numeric(1.0));
row.insert("value".into(), FieldValue::Numeric(420.0));
row.insert("region".into(), FieldValue::Text("North".into()));
let table = DataTable::from_rows(vec![row]);

// Definisci l'encoding: quale colonna → quale asse / canale colore
let enc = Encoding::new(
    Field::quantitative("month"),
    Field::quantitative("value"),
).with_color(Field::nominal("region"));

// Converti in Dataset pronto per il chart (fa groupby su "region" in automatico)
let dataset = table.to_dataset(&enc);
```

`FieldValue` supporta conversione implicita da tipi primitivi Rust:

```rust
let v: FieldValue = 3.14_f64.into();    // Numeric
let v: FieldValue = "East".into();      // Text
let v: FieldValue = true.into();        // Bool
```

### Livello 3 — CSV parser

Per caricare CSV (es. da un fetch HTTP lato browser), usa `parse_csv`:

```rust
use lodviz_core::core::csv::parse_csv;

let csv = "\
month,value,region
1,420,North
2,380,South
3,510,North";

let table = parse_csv(csv)?;
// → stessa DataTable del livello 2, poi encoding come sopra
```

Regole del parser:
- Prima riga non vuota e non-commento (`#`) → header
- Celle numeriche → `FieldValue::Numeric(f64)`
- Celle non numeriche → `FieldValue::Text`
- Celle mancanti → `FieldValue::Null`

> **Nota**: il parser accetta `&str`. Il fetch HTTP e la lettura file
> sono responsabilità dell'applicazione (nessun I/O in questo crate).

### Pipeline completa

```
CSV &str
  ──parse_csv()──▶ DataTable
                      ──to_dataset(&enc)──▶ Dataset
                                                ──lttb_downsample(n)──▶ Dataset (LOD)
                                                                             ──▶ SVG (lodviz_components)
```

## Algorithm References

- **LTTB**: Sveinn Steinarsson (2013) — *Downsampling Time Series for Visual Representation*
  ([PDF](http://skemman.is/stream/get/1946/15343/37285/3/SS_MSthesis.pdf))
- **M4**: Uwe Jugel et al. (2014) — *M4: A Visualization-Oriented Time Series Data Aggregation*

## License

MIT — see [LICENSE](../../LICENSE)

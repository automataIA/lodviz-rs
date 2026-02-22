//! Basic data structures example
//!
//! Demonstrates how to build a `Dataset` with multiple `Series` and
//! compute simple statistics over them.
//!
//! Run with:
//! ```sh
//! cargo run --example basic_data -p lodviz_core
//! ```

use lodviz_core::core::data::{DataPoint, Dataset, Series};

fn main() {
    // --- Build a multi-series dataset ---
    let sine: Vec<DataPoint> = (0..50)
        .map(|i| {
            let t = i as f64 * 0.2;
            DataPoint::new(t, t.sin())
        })
        .collect();

    let cosine: Vec<DataPoint> = (0..50)
        .map(|i| {
            let t = i as f64 * 0.2;
            DataPoint::new(t, t.cos())
        })
        .collect();

    let s1 = Series::new("sin(t)", sine);
    let s2 = Series::new("cos(t)", cosine);

    let mut dataset = Dataset::new();
    dataset.add_series(s1);
    dataset.add_series(s2);

    println!("Dataset with {} series:", dataset.series.len());
    for series in &dataset.series {
        let n = series.data.len();
        let y_sum: f64 = series.data.iter().map(|p| p.y).sum();
        let y_mean = y_sum / n as f64;
        println!(
            "  {:>10}: {} points, mean y = {:.4}",
            series.name, n, y_mean
        );
    }

    // --- Dataset from a single series ---
    let linear = Series::new(
        "linear",
        (0..10)
            .map(|i| DataPoint::new(i as f64, i as f64 * 2.0))
            .collect(),
    );
    let single = Dataset::from_series(linear);
    println!("\nSingle-series dataset: {} series", single.series.len());
    assert_eq!(single.series.len(), 1);

    // --- Visibility toggle ---
    let mut ds = Dataset::new();
    ds.add_series(Series::new("a", vec![DataPoint::new(0.0, 1.0)]));
    ds.add_series(Series::new("b", vec![DataPoint::new(1.0, 2.0)]));

    // Mark the first series as hidden
    ds.series[0].visible = false;
    let visible_count = ds.series.iter().filter(|s| s.visible).count();
    println!(
        "\nAfter hiding 'a': {} visible series (expected 1)",
        visible_count
    );
    assert_eq!(visible_count, 1);
    println!("All assertions passed ✓");
}

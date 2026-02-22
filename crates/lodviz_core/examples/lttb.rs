//! LTTB downsampling example
//!
//! Demonstrates how to reduce a large time series to a smaller number of
//! representative points while preserving the visual shape of the data.
//!
//! Run with:
//! ```sh
//! cargo run --example lttb -p lodviz_core
//! ```

use lodviz_core::algorithms::lttb::lttb_downsample;
use lodviz_core::core::data::DataPoint;

fn main() {
    // Generate a noisy sine wave with 10 000 points
    let data: Vec<DataPoint> = (0..10_000)
        .map(|i| {
            let t = i as f64 * 0.01;
            let noise = ((i * 17 + 3) % 7) as f64 * 0.05;
            DataPoint::new(t, t.sin() + noise)
        })
        .collect();

    println!("Original points:    {}", data.len());

    // Downsample to 200 points
    let reduced = lttb_downsample(&data, 200);
    println!("Downsampled points: {}", reduced.len());

    // The first and last points are always preserved
    assert_eq!(reduced.first(), data.first());
    assert_eq!(reduced.last(), data.last());
    println!("First/last points preserved: ✓");

    // Verify output length
    assert_eq!(reduced.len(), 200);
    println!("Output length = 200: ✓");

    // Show a few sampled points
    println!("\nSample of downsampled points:");
    for pt in reduced.iter().take(5) {
        println!("  x={:.3}  y={:.4}", pt.x, pt.y);
    }

    // Edge cases
    let empty: Vec<DataPoint> = vec![];
    assert!(lttb_downsample(&empty, 100).is_empty());
    println!("\nEdge case (empty input): ✓");

    let single = vec![DataPoint::new(0.0, 1.0)];
    assert_eq!(lttb_downsample(&single, 100).len(), 1);
    println!("Edge case (single point): ✓");
}

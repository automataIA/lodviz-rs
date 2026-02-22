//! Scale mapping example
//!
//! Demonstrates how to use `LinearScale` to map data values (domain) to
//! pixel positions (range), and back via the inverse transformation.
//!
//! Run with:
//! ```sh
//! cargo run --example scales -p lodviz_core
//! ```

use lodviz_core::core::scale::{LinearScale, Scale};

fn main() {
    // --- Linear scale ---
    // Map data domain [0, 100] → screen range [0, 600] (px)
    let x_scale = LinearScale::from_extent(0.0, 100.0, 0.0, 600.0);

    println!("LinearScale: domain [0, 100] → range [0, 600]");
    println!("  map(0)   = {:.1} px  (expected 0.0)", x_scale.map(0.0));
    println!("  map(50)  = {:.1} px  (expected 300.0)", x_scale.map(50.0));
    println!(
        "  map(100) = {:.1} px  (expected 600.0)",
        x_scale.map(100.0)
    );

    assert!((x_scale.map(0.0) - 0.0).abs() < 1e-9);
    assert!((x_scale.map(50.0) - 300.0).abs() < 1e-9);
    assert!((x_scale.map(100.0) - 600.0).abs() < 1e-9);
    println!("  Assertions passed ✓");

    // --- Inverse mapping ---
    // Convert a pixel position back to data value
    let data_value = x_scale.inverse(300.0);
    println!(
        "\nInverse scale: 300 px → {:.1} (expected 50.0)",
        data_value
    );
    assert!((data_value - 50.0).abs() < 1e-9);
    println!("  Assertion passed ✓");

    // --- Y scale (inverted, as SVG y-axis grows downward) ---
    // Data range [0, 1000] → screen range [400, 0] (px, inverted for SVG)
    let y_scale = LinearScale::from_extent(0.0, 1000.0, 400.0, 0.0);

    println!("\nY-axis (inverted) LinearScale: domain [0, 1000] → range [400, 0]");
    println!("  map(0)    = {:.1} px  (expected 400.0)", y_scale.map(0.0));
    println!(
        "  map(500)  = {:.1} px  (expected 200.0)",
        y_scale.map(500.0)
    );
    println!(
        "  map(1000) = {:.1} px  (expected 0.0)",
        y_scale.map(1000.0)
    );

    assert!((y_scale.map(0.0) - 400.0).abs() < 1e-9);
    assert!((y_scale.map(500.0) - 200.0).abs() < 1e-9);
    assert!((y_scale.map(1000.0) - 0.0).abs() < 1e-9);
    println!("  Assertions passed ✓");

    // --- Edge case: zero-width domain ---
    let degenerate = LinearScale::from_extent(5.0, 5.0, 0.0, 100.0);
    println!(
        "\nEdge case (zero-width domain): map(5) = {:.1}",
        degenerate.map(5.0)
    );
    // Returns range_min without panicking
    println!("  No panic ✓");
}

/// Arc geometry for pie/donut charts
use std::f64::consts::PI;

/// A computed arc slice for pie/donut charts
#[derive(Debug, Clone, PartialEq)]
pub struct ArcSlice {
    /// Start angle in radians (0 = 12 o'clock, clockwise)
    pub start_angle: f64,
    /// End angle in radians
    pub end_angle: f64,
    /// Original value
    pub value: f64,
    /// Percentage of total (0.0 to 100.0)
    pub percentage: f64,
}

impl ArcSlice {
    /// Get the mid-angle of this slice (useful for label positioning)
    pub fn mid_angle(&self) -> f64 {
        (self.start_angle + self.end_angle) / 2.0
    }
}

/// Compute arc slices from a list of values
///
/// Values are normalized to sum to 2*PI radians (full circle).
/// Starting from -PI/2 (12 o'clock position), going clockwise.
pub fn compute_arcs(values: &[f64]) -> Vec<ArcSlice> {
    let total: f64 = values.iter().filter(|v| **v > 0.0).sum();
    if total <= 0.0 {
        return vec![];
    }

    let mut arcs = Vec::with_capacity(values.len());
    let mut current_angle = -PI / 2.0; // Start at 12 o'clock

    for &value in values {
        if value <= 0.0 {
            continue;
        }
        let percentage = (value / total) * 100.0;
        let angle_span = (value / total) * 2.0 * PI;
        let end_angle = current_angle + angle_span;

        arcs.push(ArcSlice {
            start_angle: current_angle,
            end_angle,
            value,
            percentage,
        });

        current_angle = end_angle;
    }

    arcs
}

/// Generate an SVG arc path string for a pie/donut slice
///
/// - `cx`, `cy`: center of the pie
/// - `outer_r`: outer radius
/// - `inner_r`: inner radius (0 for pie, >0 for donut)
/// - `start_angle`, `end_angle`: in radians
pub fn arc_path(
    cx: f64,
    cy: f64,
    outer_r: f64,
    inner_r: f64,
    start_angle: f64,
    end_angle: f64,
) -> String {
    let angle_span = end_angle - start_angle;

    // For nearly full circle, use two arcs
    let large_arc_flag = if angle_span.abs() > PI { 1 } else { 0 };
    let sweep_flag = 1; // Clockwise

    let start_outer_x = cx + outer_r * start_angle.cos();
    let start_outer_y = cy + outer_r * start_angle.sin();
    let end_outer_x = cx + outer_r * end_angle.cos();
    let end_outer_y = cy + outer_r * end_angle.sin();

    if inner_r <= 0.0 {
        // Pie slice: move to center, line to arc start, arc, close
        format!(
            "M {cx:.2} {cy:.2} L {start_outer_x:.2} {start_outer_y:.2} A {outer_r:.2} {outer_r:.2} 0 {large_arc_flag} {sweep_flag} {end_outer_x:.2} {end_outer_y:.2} Z"
        )
    } else {
        // Donut slice: outer arc forward, line to inner, inner arc backward, close
        let start_inner_x = cx + inner_r * end_angle.cos();
        let start_inner_y = cy + inner_r * end_angle.sin();
        let end_inner_x = cx + inner_r * start_angle.cos();
        let end_inner_y = cy + inner_r * start_angle.sin();

        format!(
            "M {start_outer_x:.2} {start_outer_y:.2} A {outer_r:.2} {outer_r:.2} 0 {large_arc_flag} {sweep_flag} {end_outer_x:.2} {end_outer_y:.2} L {start_inner_x:.2} {start_inner_y:.2} A {inner_r:.2} {inner_r:.2} 0 {large_arc_flag} 0 {end_inner_x:.2} {end_inner_y:.2} Z"
        )
    }
}

/// Compute the centroid position of an arc (for label placement)
pub fn arc_centroid(cx: f64, cy: f64, radius: f64, start_angle: f64, end_angle: f64) -> (f64, f64) {
    let mid = (start_angle + end_angle) / 2.0;
    (cx + radius * mid.cos(), cy + radius * mid.sin())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6
    }

    #[test]
    fn test_compute_arcs_basic() {
        let arcs = compute_arcs(&[50.0, 50.0]);
        assert_eq!(arcs.len(), 2);
        assert!(approx_eq(arcs[0].percentage, 50.0));
        assert!(approx_eq(arcs[1].percentage, 50.0));
    }

    #[test]
    fn test_compute_arcs_single() {
        let arcs = compute_arcs(&[100.0]);
        assert_eq!(arcs.len(), 1);
        assert!(approx_eq(arcs[0].percentage, 100.0));
        // Full circle
        let span = arcs[0].end_angle - arcs[0].start_angle;
        assert!(approx_eq(span, 2.0 * PI));
    }

    #[test]
    fn test_compute_arcs_three() {
        let arcs = compute_arcs(&[25.0, 50.0, 25.0]);
        assert_eq!(arcs.len(), 3);
        assert!(approx_eq(arcs[0].percentage, 25.0));
        assert!(approx_eq(arcs[1].percentage, 50.0));
        assert!(approx_eq(arcs[2].percentage, 25.0));
    }

    #[test]
    fn test_compute_arcs_empty() {
        assert!(compute_arcs(&[]).is_empty());
    }

    #[test]
    fn test_compute_arcs_all_zero() {
        assert!(compute_arcs(&[0.0, 0.0]).is_empty());
    }

    #[test]
    fn test_compute_arcs_skip_negative() {
        let arcs = compute_arcs(&[30.0, -10.0, 70.0]);
        assert_eq!(arcs.len(), 2);
        assert!(approx_eq(arcs[0].percentage, 30.0));
        assert!(approx_eq(arcs[1].percentage, 70.0));
    }

    #[test]
    fn test_arc_path_pie() {
        let path = arc_path(100.0, 100.0, 80.0, 0.0, -PI / 2.0, 0.0);
        assert!(path.starts_with("M 100.00 100.00"));
        assert!(path.contains("A 80.00"));
        assert!(path.ends_with('Z'));
    }

    #[test]
    fn test_arc_path_donut() {
        let path = arc_path(100.0, 100.0, 80.0, 40.0, -PI / 2.0, 0.0);
        // Donut doesn't start from center
        assert!(!path.starts_with("M 100.00 100.00"));
        assert!(path.contains("A 80.00"));
        assert!(path.contains("A 40.00"));
        assert!(path.ends_with('Z'));
    }

    #[test]
    fn test_arc_centroid() {
        let (cx, cy) = arc_centroid(100.0, 100.0, 50.0, -PI / 2.0, 0.0);
        // Mid angle is -PI/4, so centroid is at (100 + 50*cos(-PI/4), 100 + 50*sin(-PI/4))
        let expected_x = 100.0 + 50.0 * (-PI / 4.0).cos();
        let expected_y = 100.0 + 50.0 * (-PI / 4.0).sin();
        assert!(approx_eq(cx, expected_x));
        assert!(approx_eq(cy, expected_y));
    }
}

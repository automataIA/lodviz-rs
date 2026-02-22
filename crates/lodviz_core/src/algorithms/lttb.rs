/// Largest-Triangle-Three-Buckets (LTTB) downsampling algorithm
///
/// LTTB is a time-series downsampling algorithm that preserves the visual
/// shape of the data by selecting points that form the largest triangles.
/// This ensures that peaks, valleys, and trends remain visible even with
/// aggressive downsampling.
///
/// Reference: Sveinn Steinarsson (2013)
/// "Downsampling Time Series for Visual Representation"
use crate::core::data::DataPoint;

/// Downsample data using the LTTB algorithm
///
/// # Arguments
///
/// * `data` - Input data points (must be sorted by x)
/// * `threshold` - Target number of points in the output
///
/// # Returns
///
/// A downsampled vector with at most `threshold` points
///
/// # Algorithm
///
/// 1. Always keep the first and last points
/// 2. Divide remaining points into (threshold - 2) buckets
/// 3. For each bucket, select the point that forms the largest
///    triangle area with the previous point and the average of the next bucket
///
/// # Examples
///
/// ```
/// use lodviz_core::core::data::DataPoint;
/// use lodviz_core::algorithms::lttb::lttb_downsample;
///
/// let data: Vec<DataPoint> = (0..1000)
///     .map(|i| DataPoint::new(i as f64, (i as f64).sin()))
///     .collect();
///
/// let downsampled = lttb_downsample(&data, 100);
/// assert_eq!(downsampled.len(), 100);
/// ```
pub fn lttb_downsample(data: &[DataPoint], threshold: usize) -> Vec<DataPoint> {
    // Edge cases
    if threshold >= data.len() || threshold == 0 {
        return data.to_vec();
    }

    if threshold == 1 {
        return vec![data[0]];
    }

    if threshold == 2 {
        return vec![data[0], data[data.len() - 1]];
    }

    // Allocate output vector
    let mut sampled = Vec::with_capacity(threshold);

    // Always include first point
    sampled.push(data[0]);

    // Bucket size (excluding first and last points)
    let bucket_size = (data.len() - 2) as f64 / (threshold - 2) as f64;

    // Initially, point_index is the index of the last selected point
    let mut point_index = 0;

    for i in 0..(threshold - 2) {
        // Calculate range for current bucket
        let avg_range_start = ((i + 1) as f64 * bucket_size + 1.0).floor() as usize;
        let avg_range_end = ((i + 2) as f64 * bucket_size + 1.0).floor() as usize;
        let avg_range_end = avg_range_end.min(data.len());

        // Calculate average point of next bucket (for triangle calculation)
        let avg_x: f64 = data[avg_range_start..avg_range_end]
            .iter()
            .map(|p| p.x)
            .sum::<f64>()
            / (avg_range_end - avg_range_start) as f64;

        let avg_y: f64 = data[avg_range_start..avg_range_end]
            .iter()
            .map(|p| p.y)
            .sum::<f64>()
            / (avg_range_end - avg_range_start) as f64;

        // Get the range for this bucket
        let range_start = ((i as f64) * bucket_size + 1.0).floor() as usize;
        let range_end = avg_range_start;

        // Point previously selected
        let point_a = data[point_index];

        let mut max_area = -1.0;
        let mut max_area_point = range_start;

        // Find the point in this bucket that forms the largest triangle
        // with point_a and the average of the next bucket
        for (offset, point_b) in data.iter().enumerate().take(range_end).skip(range_start) {
            let idx = offset;

            // Calculate triangle area using the cross product formula:
            // Area = 0.5 * |x_a(y_b - y_c) + x_b(y_c - y_a) + x_c(y_a - y_b)|
            let area = ((point_a.x - avg_x) * (point_b.y - point_a.y)
                - (point_a.x - point_b.x) * (avg_y - point_a.y))
                .abs();

            if area > max_area {
                max_area = area;
                max_area_point = idx;
            }
        }

        // Select the point with the largest triangle area
        sampled.push(data[max_area_point]);
        point_index = max_area_point;
    }

    // Always include last point
    sampled.push(data[data.len() - 1]);

    sampled
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lttb_preserves_extremes() {
        let data: Vec<DataPoint> = (0..100)
            .map(|i| DataPoint::new(i as f64, i as f64))
            .collect();

        let downsampled = lttb_downsample(&data, 10);

        // First and last points must be preserved
        assert_eq!(downsampled[0], data[0]);
        assert_eq!(downsampled[downsampled.len() - 1], data[data.len() - 1]);
    }

    #[test]
    fn test_lttb_reduces_points() {
        let data: Vec<DataPoint> = (0..1000)
            .map(|i| DataPoint::new(i as f64, (i as f64 * 0.1).sin()))
            .collect();

        let threshold = 50;
        let downsampled = lttb_downsample(&data, threshold);

        assert_eq!(downsampled.len(), threshold);
    }

    #[test]
    fn test_lttb_edge_case_zero_threshold() {
        let data: Vec<DataPoint> = (0..100)
            .map(|i| DataPoint::new(i as f64, i as f64))
            .collect();

        let downsampled = lttb_downsample(&data, 0);
        assert_eq!(downsampled.len(), data.len());
    }

    #[test]
    fn test_lttb_edge_case_one_point() {
        let data: Vec<DataPoint> = (0..100)
            .map(|i| DataPoint::new(i as f64, i as f64))
            .collect();

        let downsampled = lttb_downsample(&data, 1);
        assert_eq!(downsampled.len(), 1);
        assert_eq!(downsampled[0], data[0]);
    }

    #[test]
    fn test_lttb_edge_case_two_points() {
        let data: Vec<DataPoint> = (0..100)
            .map(|i| DataPoint::new(i as f64, i as f64))
            .collect();

        let downsampled = lttb_downsample(&data, 2);
        assert_eq!(downsampled.len(), 2);
        assert_eq!(downsampled[0], data[0]);
        assert_eq!(downsampled[1], data[data.len() - 1]);
    }

    #[test]
    fn test_lttb_threshold_larger_than_data() {
        let data: Vec<DataPoint> = (0..10)
            .map(|i| DataPoint::new(i as f64, i as f64))
            .collect();

        let downsampled = lttb_downsample(&data, 100);
        assert_eq!(downsampled.len(), data.len());
    }

    #[test]
    fn test_lttb_preserves_shape() {
        // Create a sine wave with a clear peak
        let data: Vec<DataPoint> = (0..1000)
            .map(|i| {
                let x = i as f64;
                let y = (x * 0.01).sin();
                DataPoint::new(x, y)
            })
            .collect();

        let downsampled = lttb_downsample(&data, 50);

        // Find the peak in original data
        let original_max = data.iter().map(|p| p.y).fold(f64::NEG_INFINITY, f64::max);

        // Find the peak in downsampled data
        let downsampled_max = downsampled
            .iter()
            .map(|p| p.y)
            .fold(f64::NEG_INFINITY, f64::max);

        // The peaks should be very close (within 10% tolerance for visual accuracy)
        assert!((original_max - downsampled_max).abs() / original_max < 0.1);
    }

    #[test]
    fn test_lttb_with_flat_data() {
        // All points have the same y value
        let data: Vec<DataPoint> = (0..100).map(|i| DataPoint::new(i as f64, 5.0)).collect();

        let downsampled = lttb_downsample(&data, 10);

        assert_eq!(downsampled.len(), 10);
        // All y values should still be 5.0
        assert!(downsampled.iter().all(|p| (p.y - 5.0).abs() < 1e-10));
    }

    #[test]
    fn test_lttb_monotonicity() {
        // Test with strictly increasing data
        let data: Vec<DataPoint> = (0..100)
            .map(|i| DataPoint::new(i as f64, i as f64))
            .collect();

        let downsampled = lttb_downsample(&data, 20);

        // X values should still be monotonically increasing
        for i in 1..downsampled.len() {
            assert!(downsampled[i].x > downsampled[i - 1].x);
        }
    }
}

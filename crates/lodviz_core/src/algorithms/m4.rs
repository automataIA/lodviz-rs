/// M4: Visualization-Oriented Aggregation (Jugel et al., VLDB 2014)
///
/// Selects first, last, min, and max per pixel-column bucket, guaranteeing
/// pixel-perfect rendering with at most `4 * n_pixels` output points.
use crate::core::data::DataPoint;

/// Downsample `data` to at most `4 * n_pixels` points while preserving visual fidelity.
///
/// For each pixel column bucket the algorithm emits up to 4 points:
/// the first, last, minimum-y, and maximum-y point in that bucket.
/// Duplicate points within a bucket are de-duplicated.
///
/// Returns `data.to_vec()` unchanged when `data.len() <= 4 * n_pixels`.
pub fn m4_downsample(data: &[DataPoint], n_pixels: usize) -> Vec<DataPoint> {
    if data.is_empty() || n_pixels == 0 {
        return vec![];
    }
    if data.len() <= 4 * n_pixels {
        return data.to_vec();
    }

    let x_min = data[0].x;
    let x_max = data[data.len() - 1].x;
    let x_range = x_max - x_min;

    if x_range <= 0.0 {
        // All points at the same x — nothing to aggregate further
        return data.to_vec();
    }

    let bucket_width = x_range / n_pixels as f64;
    let mut result: Vec<DataPoint> = Vec::with_capacity(4 * n_pixels);

    for bucket_idx in 0..n_pixels {
        let bucket_start = x_min + bucket_idx as f64 * bucket_width;
        let bucket_end = bucket_start + bucket_width;

        // Include right-endpoint for the last bucket
        let in_bucket = |p: &&DataPoint| {
            p.x >= bucket_start && (p.x < bucket_end || bucket_idx == n_pixels - 1)
        };

        let bucket_points: Vec<&DataPoint> = data.iter().filter(in_bucket).collect();

        if bucket_points.is_empty() {
            continue;
        }

        let first = *bucket_points[0];
        let last = *bucket_points[bucket_points.len() - 1];

        let min_pt = *bucket_points
            .iter()
            .copied()
            .min_by(|a, b| a.y.partial_cmp(&b.y).unwrap_or(std::cmp::Ordering::Equal))
            .expect("bucket is non-empty");

        let max_pt = *bucket_points
            .iter()
            .copied()
            .max_by(|a, b| a.y.partial_cmp(&b.y).unwrap_or(std::cmp::Ordering::Equal))
            .expect("bucket is non-empty");

        // Collect unique points for this bucket, ordered by x
        let mut pts = [first, last, min_pt, max_pt];
        pts.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal));

        // Push with de-duplication by (x, y) equality
        for pt in pts {
            if result
                .last()
                .map(|prev: &DataPoint| {
                    (prev.x - pt.x).abs() < f64::EPSILON && (prev.y - pt.y).abs() < f64::EPSILON
                })
                .unwrap_or(false)
            {
                continue;
            }
            result.push(pt);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_points(n: usize) -> Vec<DataPoint> {
        (0..n)
            .map(|i| DataPoint::new(i as f64, (i as f64 * 0.1).sin()))
            .collect()
    }

    #[test]
    fn test_m4_passthrough_small_data() {
        let data = make_points(40);
        let result = m4_downsample(&data, 10); // 4*10=40 == data.len()
        assert_eq!(result.len(), data.len());
    }

    #[test]
    fn test_m4_reduces_large_data() {
        let data = make_points(10_000);
        let n_pixels = 200;
        let result = m4_downsample(&data, n_pixels);
        assert!(result.len() <= 4 * n_pixels + 4); // slight slack for edge buckets
        assert!(!result.is_empty());
    }

    #[test]
    fn test_m4_preserves_order() {
        let data = make_points(1000);
        let result = m4_downsample(&data, 50);
        for w in result.windows(2) {
            assert!(
                w[0].x <= w[1].x,
                "Output not sorted: {:.3} > {:.3}",
                w[0].x,
                w[1].x
            );
        }
    }

    #[test]
    fn test_m4_empty_input() {
        assert!(m4_downsample(&[], 100).is_empty());
    }

    #[test]
    fn test_m4_zero_pixels() {
        let data = make_points(100);
        assert!(m4_downsample(&data, 0).is_empty());
    }
}

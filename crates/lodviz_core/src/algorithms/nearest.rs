/// Nearest-point search for tooltip interaction
use crate::core::data::DataPoint;

/// Find the DataPoint with x closest to `target_x`.
/// Assumes data is sorted by x. Uses binary search for O(log n).
/// Returns the index and a reference to the nearest point.
pub fn find_nearest_point(data: &[DataPoint], target_x: f64) -> Option<(usize, &DataPoint)> {
    if data.is_empty() {
        return None;
    }

    let idx = data.partition_point(|p| p.x < target_x);

    // Compare the candidates on either side of the partition point
    let candidates: &[usize] = match idx {
        0 => &[0],
        i if i >= data.len() => &[data.len() - 1],
        i => &[i - 1, i],
    };

    candidates
        .iter()
        .copied()
        .min_by(|&a, &b| {
            let da = (data[a].x - target_x).abs();
            let db = (data[b].x - target_x).abs();
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|i| (i, &data[i]))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_data() -> Vec<DataPoint> {
        vec![
            DataPoint::new(0.0, 10.0),
            DataPoint::new(1.0, 20.0),
            DataPoint::new(2.0, 15.0),
            DataPoint::new(5.0, 30.0),
            DataPoint::new(10.0, 5.0),
        ]
    }

    #[test]
    fn test_nearest_exact_match() {
        let data = sample_data();
        let (idx, pt) = find_nearest_point(&data, 2.0).unwrap();
        assert_eq!(idx, 2);
        assert_eq!(pt.x, 2.0);
        assert_eq!(pt.y, 15.0);
    }

    #[test]
    fn test_nearest_between_points() {
        let data = sample_data();
        // 4.0 is closer to 5.0 than to 2.0
        let (idx, pt) = find_nearest_point(&data, 4.0).unwrap();
        assert_eq!(idx, 3);
        assert_eq!(pt.x, 5.0);
    }

    #[test]
    fn test_nearest_before_first() {
        let data = sample_data();
        let (idx, _) = find_nearest_point(&data, -5.0).unwrap();
        assert_eq!(idx, 0);
    }

    #[test]
    fn test_nearest_after_last() {
        let data = sample_data();
        let (idx, _) = find_nearest_point(&data, 100.0).unwrap();
        assert_eq!(idx, 4);
    }

    #[test]
    fn test_nearest_empty() {
        assert!(find_nearest_point(&[], 5.0).is_none());
    }

    #[test]
    fn test_nearest_single_point() {
        let data = vec![DataPoint::new(3.0, 7.0)];
        let (idx, pt) = find_nearest_point(&data, 100.0).unwrap();
        assert_eq!(idx, 0);
        assert_eq!(pt.x, 3.0);
    }

    #[test]
    fn test_nearest_midpoint_prefers_left() {
        let data = vec![DataPoint::new(0.0, 0.0), DataPoint::new(2.0, 0.0)];
        // At exact midpoint (1.0), both are equidistant. Either is acceptable.
        let (idx, _) = find_nearest_point(&data, 1.0).unwrap();
        assert!(idx == 0 || idx == 1);
    }
}

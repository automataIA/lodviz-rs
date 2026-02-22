/// Selection model for interactive chart features
///
/// Represents user selections on charts, inspired by Vega-Lite selection types.
use crate::core::data::DataPoint;

/// A selection on chart data
#[derive(Debug, Clone, PartialEq)]
pub enum Selection {
    /// Selection of specific data points by index
    Point {
        /// The indices of the selected items
        indices: Vec<usize>,
    },
    /// Selection of a continuous interval range
    Interval {
        /// The bounds on the X axis `(min, max)`
        x: (f64, f64),
        /// The optional bounds on the Y axis `(min, max)`
        y: Option<(f64, f64)>,
    },
    /// Combination of multiple selections
    Multi {
        /// A collection of active selections
        selections: Vec<Selection>,
    },
}

impl Selection {
    /// Create a point selection from indices
    pub fn point(indices: Vec<usize>) -> Self {
        Self::Point { indices }
    }

    /// Create an x-axis interval selection
    pub fn interval_x(x_min: f64, x_max: f64) -> Self {
        let (x_min, x_max) = if x_min <= x_max {
            (x_min, x_max)
        } else {
            (x_max, x_min)
        };
        Self::Interval {
            x: (x_min, x_max),
            y: None,
        }
    }

    /// Create a 2D interval selection
    pub fn interval_xy(x_min: f64, x_max: f64, y_min: f64, y_max: f64) -> Self {
        let (x_min, x_max) = if x_min <= x_max {
            (x_min, x_max)
        } else {
            (x_max, x_min)
        };
        let (y_min, y_max) = if y_min <= y_max {
            (y_min, y_max)
        } else {
            (y_max, y_min)
        };
        Self::Interval {
            x: (x_min, x_max),
            y: Some((y_min, y_max)),
        }
    }

    /// Check if an empty selection
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Point { indices } => indices.is_empty(),
            Self::Interval { x, .. } => (x.1 - x.0).abs() < f64::EPSILON,
            Self::Multi { selections } => selections.is_empty(),
        }
    }

    /// Check if a data point is within this selection
    pub fn contains_point(&self, point: &DataPoint, index: usize) -> bool {
        match self {
            Self::Point { indices } => indices.contains(&index),
            Self::Interval { x, y } => {
                let in_x = point.x >= x.0 && point.x <= x.1;
                let in_y = y.is_none_or(|(y_min, y_max)| point.y >= y_min && point.y <= y_max);
                in_x && in_y
            }
            Self::Multi { selections } => selections.iter().any(|s| s.contains_point(point, index)),
        }
    }
}

/// Filter data points by a selection
///
/// Returns a new vector containing only the points that match the selection.
pub fn filter_by_selection(data: &[DataPoint], selection: &Selection) -> Vec<DataPoint> {
    data.iter()
        .enumerate()
        .filter(|(i, p)| selection.contains_point(p, *i))
        .map(|(_, p)| *p)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_data() -> Vec<DataPoint> {
        vec![
            DataPoint::new(1.0, 10.0),
            DataPoint::new(2.0, 20.0),
            DataPoint::new(3.0, 30.0),
            DataPoint::new(4.0, 40.0),
            DataPoint::new(5.0, 50.0),
        ]
    }

    #[test]
    fn test_point_selection() {
        let sel = Selection::point(vec![1, 3]);
        let data = sample_data();
        assert!(sel.contains_point(&data[1], 1));
        assert!(sel.contains_point(&data[3], 3));
        assert!(!sel.contains_point(&data[0], 0));
    }

    #[test]
    fn test_interval_x_selection() {
        let sel = Selection::interval_x(2.0, 4.0);
        let data = sample_data();
        assert!(!sel.contains_point(&data[0], 0)); // x=1.0
        assert!(sel.contains_point(&data[1], 1)); // x=2.0
        assert!(sel.contains_point(&data[2], 2)); // x=3.0
        assert!(sel.contains_point(&data[3], 3)); // x=4.0
        assert!(!sel.contains_point(&data[4], 4)); // x=5.0
    }

    #[test]
    fn test_interval_xy_selection() {
        let sel = Selection::interval_xy(2.0, 4.0, 15.0, 35.0);
        let data = sample_data();
        assert!(!sel.contains_point(&data[0], 0)); // x=1 out of x range
        assert!(sel.contains_point(&data[1], 1)); // x=2, y=20 in range
        assert!(sel.contains_point(&data[2], 2)); // x=3, y=30 in range
        assert!(!sel.contains_point(&data[3], 3)); // x=4, y=40 out of y range
    }

    #[test]
    fn test_interval_auto_sort() {
        // If min > max, should auto-sort
        let sel = Selection::interval_x(5.0, 2.0);
        let data = sample_data();
        assert!(sel.contains_point(&data[2], 2)); // x=3.0 within [2,5]
    }

    #[test]
    fn test_filter_by_selection() {
        let data = sample_data();
        let sel = Selection::interval_x(2.0, 4.0);
        let filtered = filter_by_selection(&data, &sel);
        assert_eq!(filtered.len(), 3);
        assert_eq!(filtered[0].x, 2.0);
        assert_eq!(filtered[1].x, 3.0);
        assert_eq!(filtered[2].x, 4.0);
    }

    #[test]
    fn test_multi_selection() {
        let sel = Selection::Multi {
            selections: vec![Selection::point(vec![0]), Selection::interval_x(4.0, 5.0)],
        };
        let data = sample_data();
        assert!(sel.contains_point(&data[0], 0));
        assert!(!sel.contains_point(&data[1], 1));
        assert!(sel.contains_point(&data[3], 3));
        assert!(sel.contains_point(&data[4], 4));
    }

    #[test]
    fn test_empty_selection() {
        assert!(Selection::point(vec![]).is_empty());
        assert!(Selection::interval_x(3.0, 3.0).is_empty());
        assert!(Selection::Multi { selections: vec![] }.is_empty());
    }

    #[test]
    fn test_filter_empty_result() {
        let data = sample_data();
        let sel = Selection::interval_x(100.0, 200.0);
        let filtered = filter_by_selection(&data, &sel);
        assert!(filtered.is_empty());
    }
}

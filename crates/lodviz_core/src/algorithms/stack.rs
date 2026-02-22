// Stacking algorithm for stacked bar/area charts

/// A single stacked entry with baseline and top value
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StackedValue {
    /// The bottom of this segment (cumulative sum of previous series)
    pub y0: f64,
    /// The top of this segment (y0 + this series value)
    pub y1: f64,
}

/// A stacked series containing the original series index and computed stack values
#[derive(Debug, Clone)]
pub struct StackedSeries {
    /// The original index of this series in the un-stacked dataset
    pub series_index: usize,
    /// The computed stacked values (y0 and y1) for each category
    pub values: Vec<StackedValue>,
}

/// Compute stacked values from multiple series
///
/// Each inner `Vec<f64>` represents one series of values aligned by category index.
/// All series must have the same length (one value per category).
///
/// Returns one `StackedSeries` per input series, with cumulative y0/y1 values.
pub fn stack_series(series_values: &[Vec<f64>]) -> Vec<StackedSeries> {
    if series_values.is_empty() {
        return vec![];
    }

    let n_categories = series_values[0].len();
    let mut result = Vec::with_capacity(series_values.len());
    let mut baselines = vec![0.0_f64; n_categories];

    for (si, values) in series_values.iter().enumerate() {
        let stacked: Vec<StackedValue> = values
            .iter()
            .enumerate()
            .map(|(ci, &v)| {
                let y0 = baselines.get(ci).copied().unwrap_or(0.0);
                let y1 = y0 + v;
                StackedValue { y0, y1 }
            })
            .collect();

        // Update baselines
        for (ci, sv) in stacked.iter().enumerate() {
            if let Some(b) = baselines.get_mut(ci) {
                *b = sv.y1;
            }
        }

        result.push(StackedSeries {
            series_index: si,
            values: stacked,
        });
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack_single_series() {
        let input = vec![vec![10.0, 20.0, 30.0]];
        let result = stack_series(&input);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].values[0], StackedValue { y0: 0.0, y1: 10.0 });
        assert_eq!(result[0].values[1], StackedValue { y0: 0.0, y1: 20.0 });
        assert_eq!(result[0].values[2], StackedValue { y0: 0.0, y1: 30.0 });
    }

    #[test]
    fn test_stack_two_series() {
        let input = vec![vec![10.0, 20.0], vec![5.0, 15.0]];
        let result = stack_series(&input);
        assert_eq!(result.len(), 2);
        // First series: starts at 0
        assert_eq!(result[0].values[0], StackedValue { y0: 0.0, y1: 10.0 });
        assert_eq!(result[0].values[1], StackedValue { y0: 0.0, y1: 20.0 });
        // Second series: starts where first ended
        assert_eq!(result[1].values[0], StackedValue { y0: 10.0, y1: 15.0 });
        assert_eq!(result[1].values[1], StackedValue { y0: 20.0, y1: 35.0 });
    }

    #[test]
    fn test_stack_three_series() {
        let input = vec![vec![1.0, 2.0], vec![3.0, 4.0], vec![5.0, 6.0]];
        let result = stack_series(&input);
        assert_eq!(result[2].values[0], StackedValue { y0: 4.0, y1: 9.0 });
        assert_eq!(result[2].values[1], StackedValue { y0: 6.0, y1: 12.0 });
    }

    #[test]
    fn test_stack_empty() {
        let result = stack_series(&[]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_stack_zeros() {
        let input = vec![vec![0.0, 0.0], vec![5.0, 10.0]];
        let result = stack_series(&input);
        assert_eq!(result[0].values[0], StackedValue { y0: 0.0, y1: 0.0 });
        assert_eq!(result[1].values[0], StackedValue { y0: 0.0, y1: 5.0 });
    }
}

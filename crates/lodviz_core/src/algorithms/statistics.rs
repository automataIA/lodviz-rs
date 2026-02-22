/// Statistical utility functions for data analysis
///
/// Calculate the extent (min, max) of a dataset
///
/// Returns `None` if the slice is empty.
pub fn extent(data: &[f64]) -> Option<(f64, f64)> {
    if data.is_empty() {
        return None;
    }

    let mut min = data[0];
    let mut max = data[0];

    for &value in data.iter().skip(1) {
        if value < min {
            min = value;
        }
        if value > max {
            max = value;
        }
    }

    Some((min, max))
}

/// Calculate the mean (average) of a dataset
///
/// Returns `None` if the slice is empty.
pub fn mean(data: &[f64]) -> Option<f64> {
    if data.is_empty() {
        return None;
    }

    let sum: f64 = data.iter().sum();
    Some(sum / data.len() as f64)
}

/// Calculate the median of a dataset
///
/// Returns `None` if the slice is empty.
/// Note: This function sorts the data, so it modifies the input vector.
pub fn median(data: &mut [f64]) -> Option<f64> {
    if data.is_empty() {
        return None;
    }

    data.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let mid = data.len() / 2;
    if data.len().is_multiple_of(2) {
        Some((data[mid - 1] + data[mid]) / 2.0)
    } else {
        Some(data[mid])
    }
}

/// Calculate the standard deviation of a dataset
///
/// Returns `None` if the slice has fewer than 2 elements.
pub fn std_dev(data: &[f64]) -> Option<f64> {
    if data.len() < 2 {
        return None;
    }

    let mean_val = mean(data)?;
    let variance: f64 =
        data.iter().map(|&x| (x - mean_val).powi(2)).sum::<f64>() / (data.len() - 1) as f64;

    Some(variance.sqrt())
}

/// Calculate the sum of a dataset
pub fn sum(data: &[f64]) -> f64 {
    data.iter().sum()
}

// ─── New statistical types and functions ──────────────────────────────────────

/// Statistics for a single box plot group
#[derive(Debug, Clone, PartialEq)]
pub struct BoxPlotStats {
    /// The first quartile (25th percentile)
    pub q1: f64,
    /// The median (50th percentile)
    pub median: f64,
    /// The third quartile (75th percentile)
    pub q3: f64,
    /// The interquartile range (IQR)
    pub iqr: f64,
    /// The lower bounding value (Q1 - 1.5 * IQR)
    pub lower_whisker: f64,
    /// The upper bounding value (Q3 + 1.5 * IQR)
    pub upper_whisker: f64,
    /// The average value
    pub mean: f64,
    /// Identified outlier values beyond the whiskers
    pub outliers: Vec<f64>,
}

/// Rule for determining histogram bin count
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BinRule {
    /// Sturges: k = ceil(log2(n) + 1)
    Sturges,
    /// Scott: h = 3.49 * σ * n^(-1/3)
    Scott,
    /// Freedman-Diaconis: h = 2 * IQR * n^(-1/3) — robust to outliers
    #[default]
    FreedmanDiaconis,
    /// Fixed number of bins
    Fixed(usize),
}

/// A single histogram bin
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bin {
    /// Lower bound of the bin
    pub x0: f64,
    /// Upper bound of the bin
    pub x1: f64,
    /// Number of elements within this bin
    pub count: usize,
}

/// Result of kernel density estimation
#[derive(Debug, Clone)]
pub struct KdeResult {
    /// X values (data domain)
    pub xs: Vec<f64>,
    /// Y values (density estimates)
    pub ys: Vec<f64>,
}

/// Compute a percentile via linear interpolation on sorted data.
///
/// `p` must be in `[0.0, 1.0]`. Data must already be sorted ascending.
fn percentile_sorted(sorted: &[f64], p: f64) -> f64 {
    let n = sorted.len();
    if n == 0 {
        return 0.0;
    }
    if n == 1 {
        return sorted[0];
    }
    let h = p * (n - 1) as f64;
    let lo = h.floor() as usize;
    let frac = h - lo as f64;
    if lo + 1 >= n {
        sorted[n - 1]
    } else {
        sorted[lo] * (1.0 - frac) + sorted[lo + 1] * frac
    }
}

/// Compute box plot statistics, sorting `data` in-place.
///
/// Returns `None` if `data` is empty.
pub fn box_plot_stats(data: &mut [f64]) -> Option<BoxPlotStats> {
    if data.is_empty() {
        return None;
    }
    data.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let q1 = percentile_sorted(data, 0.25);
    let med = percentile_sorted(data, 0.5);
    let q3 = percentile_sorted(data, 0.75);
    let iqr = q3 - q1;
    let lower_fence = q1 - 1.5 * iqr;
    let upper_fence = q3 + 1.5 * iqr;

    let lower_whisker = data
        .iter()
        .copied()
        .find(|&x| x >= lower_fence)
        .unwrap_or(q1);
    let upper_whisker = data
        .iter()
        .rev()
        .copied()
        .find(|&x| x <= upper_fence)
        .unwrap_or(q3);

    let mean_val = data.iter().sum::<f64>() / data.len() as f64;
    let outliers: Vec<f64> = data
        .iter()
        .copied()
        .filter(|&x| x < lower_fence || x > upper_fence)
        .collect();

    Some(BoxPlotStats {
        q1,
        median: med,
        q3,
        iqr,
        lower_whisker,
        upper_whisker,
        mean: mean_val,
        outliers,
    })
}

/// Compute histogram bins for `data` using the specified `rule`.
///
/// Returns an empty `Vec` if `data` is empty.
pub fn histogram_bins(data: &[f64], rule: BinRule) -> Vec<Bin> {
    if data.is_empty() {
        return vec![];
    }
    let n = data.len();
    let (min_val, max_val) = extent(data).expect("data is non-empty");

    if (max_val - min_val).abs() < f64::EPSILON {
        // All identical → single bin covering [min, min+1)
        return vec![Bin {
            x0: min_val,
            x1: min_val + 1.0,
            count: n,
        }];
    }

    let k = match rule {
        BinRule::Sturges => {
            let k = (n as f64).log2().ceil() as usize + 1;
            k.max(1)
        }
        BinRule::Scott => {
            if let Some(sd) = std_dev(data) {
                if sd > 0.0 {
                    let h = 3.49 * sd * (n as f64).powf(-1.0 / 3.0);
                    ((max_val - min_val) / h).ceil() as usize
                } else {
                    1
                }
            } else {
                1
            }
        }
        BinRule::FreedmanDiaconis => {
            let mut sorted = data.to_vec();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let q1 = percentile_sorted(&sorted, 0.25);
            let q3 = percentile_sorted(&sorted, 0.75);
            let iqr = q3 - q1;
            if iqr > 0.0 {
                let h = 2.0 * iqr * (n as f64).powf(-1.0 / 3.0);
                ((max_val - min_val) / h).ceil() as usize
            } else {
                // Fallback to Sturges when IQR is zero
                (n as f64).log2().ceil() as usize + 1
            }
        }
        BinRule::Fixed(k) => k,
    };
    let k = k.max(1);

    let bin_width = (max_val - min_val) / k as f64;
    let mut bins: Vec<Bin> = (0..k)
        .map(|i| Bin {
            x0: min_val + i as f64 * bin_width,
            x1: min_val + (i + 1) as f64 * bin_width,
            count: 0,
        })
        .collect();

    for &val in data {
        if val < min_val || val > max_val {
            continue;
        }
        let idx = ((val - min_val) / bin_width).floor() as usize;
        let idx = idx.min(k - 1); // clamp last value into the last bin
        bins[idx].count += 1;
    }
    bins
}

/// Estimate density using a Gaussian kernel (Silverman's bandwidth rule).
///
/// `n_points` is the number of evaluation points on the grid.
/// Returns `None` if `data` has fewer than 2 elements.
pub fn gaussian_kde(data: &[f64], n_points: usize) -> Option<KdeResult> {
    if data.len() < 2 || n_points == 0 {
        return None;
    }
    let sd = std_dev(data)?;
    if sd <= 0.0 {
        return None;
    }

    let n = data.len() as f64;
    let h = 1.06 * sd * n.powf(-0.2); // Silverman's rule of thumb
    let (min_val, max_val) = extent(data)?;

    let x_lo = min_val - 3.0 * h;
    let x_hi = max_val + 3.0 * h;
    let norm_factor = 1.0 / (h * (2.0 * std::f64::consts::PI).sqrt() * n);

    let xs: Vec<f64> = (0..n_points)
        .map(|i| x_lo + (x_hi - x_lo) * i as f64 / (n_points - 1).max(1) as f64)
        .collect();

    let ys: Vec<f64> = xs
        .iter()
        .map(|&x| {
            data.iter()
                .map(|&xi| {
                    let z = (x - xi) / h;
                    norm_factor * (-0.5 * z * z).exp()
                })
                .sum::<f64>()
        })
        .collect();

    Some(KdeResult { xs, ys })
}

/// Simple Moving Average.
///
/// Returns a `Vec` of length `data.len() - window + 1`.
/// Returns empty if `window == 0` or `window > data.len()`.
pub fn sma(data: &[f64], window: usize) -> Vec<f64> {
    if window == 0 || window > data.len() {
        return vec![];
    }
    data.windows(window)
        .map(|w| w.iter().sum::<f64>() / window as f64)
        .collect()
}

/// Exponential Moving Average.
///
/// `alpha` is the smoothing factor clamped to `[0.0, 1.0]`.
/// Returns a `Vec` of the same length as `data`.
pub fn ema(data: &[f64], alpha: f64) -> Vec<f64> {
    if data.is_empty() {
        return vec![];
    }
    let alpha = alpha.clamp(0.0, 1.0);
    let mut result = Vec::with_capacity(data.len());
    result.push(data[0]);
    for i in 1..data.len() {
        let prev = result[i - 1];
        result.push(alpha * data[i] + (1.0 - alpha) * prev);
    }
    result
}

/// Ordinary Least Squares linear regression.
///
/// Returns `Some((β0, β1))` where `y = β0 + β1 * x`, or `None` if fewer than
/// 2 points or if all X values are identical.
pub fn linear_regression(points: &[(f64, f64)]) -> Option<(f64, f64)> {
    let n = points.len();
    if n < 2 {
        return None;
    }
    let n_f = n as f64;
    let x_mean = points.iter().map(|(x, _)| x).sum::<f64>() / n_f;
    let y_mean = points.iter().map(|(_, y)| y).sum::<f64>() / n_f;
    let num: f64 = points
        .iter()
        .map(|(x, y)| (x - x_mean) * (y - y_mean))
        .sum();
    let den: f64 = points.iter().map(|(x, _)| (x - x_mean).powi(2)).sum();
    if den.abs() < f64::EPSILON {
        return None;
    }
    let beta1 = num / den;
    let beta0 = y_mean - beta1 * x_mean;
    Some((beta0, beta1))
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f64 = 1e-10;

    #[test]
    fn test_extent_basic() {
        let data = vec![1.0, 5.0, 3.0, 9.0, 2.0];
        let (min, max) = extent(&data).unwrap();
        assert!((min - 1.0).abs() < EPSILON);
        assert!((max - 9.0).abs() < EPSILON);
    }

    #[test]
    fn test_extent_empty() {
        let data: Vec<f64> = vec![];
        assert!(extent(&data).is_none());
    }

    #[test]
    fn test_extent_single() {
        let data = vec![42.0];
        let (min, max) = extent(&data).unwrap();
        assert!((min - 42.0).abs() < EPSILON);
        assert!((max - 42.0).abs() < EPSILON);
    }

    #[test]
    fn test_extent_negative() {
        let data = vec![-10.0, -5.0, -20.0, -1.0];
        let (min, max) = extent(&data).unwrap();
        assert!((min + 20.0).abs() < EPSILON);
        assert!((max + 1.0).abs() < EPSILON);
    }

    #[test]
    fn test_mean_basic() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let avg = mean(&data).unwrap();
        assert!((avg - 3.0).abs() < EPSILON);
    }

    #[test]
    fn test_mean_empty() {
        let data: Vec<f64> = vec![];
        assert!(mean(&data).is_none());
    }

    #[test]
    fn test_mean_single() {
        let data = vec![42.0];
        let avg = mean(&data).unwrap();
        assert!((avg - 42.0).abs() < EPSILON);
    }

    #[test]
    fn test_median_odd_count() {
        let mut data = vec![1.0, 3.0, 5.0, 7.0, 9.0];
        let med = median(&mut data).unwrap();
        assert!((med - 5.0).abs() < EPSILON);
    }

    #[test]
    fn test_median_even_count() {
        let mut data = vec![1.0, 2.0, 3.0, 4.0];
        let med = median(&mut data).unwrap();
        assert!((med - 2.5).abs() < EPSILON);
    }

    #[test]
    fn test_median_unsorted() {
        let mut data = vec![9.0, 1.0, 5.0, 3.0, 7.0];
        let med = median(&mut data).unwrap();
        assert!((med - 5.0).abs() < EPSILON);
    }

    #[test]
    fn test_median_empty() {
        let mut data: Vec<f64> = vec![];
        assert!(median(&mut data).is_none());
    }

    #[test]
    fn test_std_dev_basic() {
        let data = vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let sd = std_dev(&data).unwrap();
        // Expected std dev ≈ 2.0
        assert!((sd - 2.138).abs() < 0.01);
    }

    #[test]
    fn test_std_dev_insufficient_data() {
        let data = vec![5.0];
        assert!(std_dev(&data).is_none());
    }

    #[test]
    fn test_std_dev_zero_variance() {
        let data = vec![5.0, 5.0, 5.0, 5.0];
        let sd = std_dev(&data).unwrap();
        assert!(sd.abs() < EPSILON);
    }

    #[test]
    fn test_sum_basic() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let total = sum(&data);
        assert!((total - 15.0).abs() < EPSILON);
    }

    #[test]
    fn test_sum_empty() {
        let data: Vec<f64> = vec![];
        let total = sum(&data);
        assert!(total.abs() < EPSILON);
    }

    #[test]
    fn test_sum_negative() {
        let data = vec![-1.0, -2.0, -3.0];
        let total = sum(&data);
        assert!((total + 6.0).abs() < EPSILON);
    }

    // ─── box_plot_stats ───────────────────────────────────────────────────────

    #[test]
    fn test_box_plot_basic() {
        let mut data: Vec<f64> = (1..=9).map(|i| i as f64).collect();
        let stats = box_plot_stats(&mut data).unwrap();
        // Q1=3, median=5, Q3=7 for [1,2,3,4,5,6,7,8,9]
        assert!((stats.median - 5.0).abs() < 0.01);
        assert!((stats.q1 - 3.0).abs() < 0.01);
        assert!((stats.q3 - 7.0).abs() < 0.01);
        assert!((stats.iqr - 4.0).abs() < 0.01);
    }

    #[test]
    fn test_box_plot_empty() {
        let mut data: Vec<f64> = vec![];
        assert!(box_plot_stats(&mut data).is_none());
    }

    #[test]
    fn test_box_plot_outliers_detected() {
        // 100 is a clear outlier from a [1..10] distribution
        let mut data: Vec<f64> = (1..=10)
            .map(|i| i as f64)
            .chain(std::iter::once(100.0))
            .collect();
        let stats = box_plot_stats(&mut data).unwrap();
        assert!(stats.outliers.contains(&100.0));
    }

    // ─── histogram_bins ───────────────────────────────────────────────────────

    #[test]
    fn test_histogram_fixed_bins() {
        let data: Vec<f64> = (0..100).map(|i| i as f64).collect();
        let bins = histogram_bins(&data, BinRule::Fixed(10));
        assert_eq!(bins.len(), 10);
        let total: usize = bins.iter().map(|b| b.count).sum();
        assert_eq!(total, 100);
    }

    #[test]
    fn test_histogram_empty() {
        let bins = histogram_bins(&[], BinRule::Fixed(5));
        assert!(bins.is_empty());
    }

    #[test]
    fn test_histogram_sturges_range() {
        // 100 points → Sturges: ~8 bins
        let data: Vec<f64> = (0..100).map(|i| i as f64).collect();
        let bins = histogram_bins(&data, BinRule::Sturges);
        assert!(!bins.is_empty());
        let total: usize = bins.iter().map(|b| b.count).sum();
        assert_eq!(total, data.len());
    }

    // ─── gaussian_kde ─────────────────────────────────────────────────────────

    #[test]
    fn test_kde_basic() {
        let data = vec![0.0, 1.0, 2.0, 3.0, 4.0];
        let kde = gaussian_kde(&data, 50).unwrap();
        assert_eq!(kde.xs.len(), 50);
        assert_eq!(kde.ys.len(), 50);
        // All densities must be non-negative
        assert!(kde.ys.iter().all(|&y| y >= 0.0));
    }

    #[test]
    fn test_kde_too_few_points() {
        let data = vec![1.0];
        assert!(gaussian_kde(&data, 50).is_none());
    }

    #[test]
    fn test_kde_peak_near_data_center() {
        // Symmetric data: peak density should be near the center
        let data: Vec<f64> = (0..11).map(|i| i as f64 - 5.0).collect();
        let kde = gaussian_kde(&data, 101).unwrap();
        let max_idx = kde
            .ys
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(i, _)| i)
            .unwrap();
        let peak_x = kde.xs[max_idx];
        assert!(peak_x.abs() < 1.0, "Peak should be near x=0, got {peak_x}");
    }

    // ─── sma ─────────────────────────────────────────────────────────────────

    #[test]
    fn test_sma_basic() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = sma(&data, 3);
        assert_eq!(result.len(), 3);
        assert!((result[0] - 2.0).abs() < EPSILON);
        assert!((result[1] - 3.0).abs() < EPSILON);
        assert!((result[2] - 4.0).abs() < EPSILON);
    }

    #[test]
    fn test_sma_window_too_large() {
        let data = vec![1.0, 2.0];
        assert!(sma(&data, 5).is_empty());
    }

    #[test]
    fn test_sma_window_one() {
        let data = vec![3.0, 5.0, 7.0];
        let result = sma(&data, 1);
        assert_eq!(result, data);
    }

    // ─── ema ─────────────────────────────────────────────────────────────────

    #[test]
    fn test_ema_basic() {
        let data = vec![1.0, 2.0, 3.0];
        let result = ema(&data, 0.5);
        assert_eq!(result.len(), 3);
        assert!((result[0] - 1.0).abs() < EPSILON);
        // result[1] = 0.5*2 + 0.5*1 = 1.5
        assert!((result[1] - 1.5).abs() < EPSILON);
    }

    #[test]
    fn test_ema_empty() {
        assert!(ema(&[], 0.5).is_empty());
    }

    #[test]
    fn test_ema_alpha_zero_preserves_first() {
        // alpha=0 → never updates, stays at first value
        let data = vec![10.0, 20.0, 30.0];
        let result = ema(&data, 0.0);
        assert!(result.iter().all(|&v| (v - 10.0).abs() < EPSILON));
    }

    // ─── linear_regression ───────────────────────────────────────────────────

    #[test]
    fn test_regression_perfect_line() {
        // y = 2x + 1
        let points: Vec<(f64, f64)> = (0..5).map(|i| (i as f64, 2.0 * i as f64 + 1.0)).collect();
        let (b0, b1) = linear_regression(&points).unwrap();
        assert!((b0 - 1.0).abs() < 0.01, "intercept = {b0}");
        assert!((b1 - 2.0).abs() < 0.01, "slope = {b1}");
    }

    #[test]
    fn test_regression_too_few_points() {
        let points = vec![(1.0, 2.0)];
        assert!(linear_regression(&points).is_none());
    }

    #[test]
    fn test_regression_vertical_line() {
        // Identical X values → undefined slope
        let points = vec![(1.0, 0.0), (1.0, 1.0), (1.0, 2.0)];
        assert!(linear_regression(&points).is_none());
    }
}

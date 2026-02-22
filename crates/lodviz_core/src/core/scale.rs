/// Scale functions for mapping data domain to visual range
///
/// Scales are pure functions that convert data values (domain) to
/// visual coordinates (range), essential for all chart types.
use std::fmt::Debug;

/// Trait for scale transformations
pub trait Scale: Debug {
    /// Map a value from domain to range
    fn map(&self, value: f64) -> f64;

    /// Inverse map: from range back to domain
    fn inverse(&self, mapped: f64) -> f64;

    /// Get the domain (input range)
    fn domain(&self) -> (f64, f64);

    /// Get the range (output range)
    fn range(&self) -> (f64, f64);
}

/// Linear scale for continuous numerical data
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LinearScale {
    domain: (f64, f64),
    range: (f64, f64),
}

impl LinearScale {
    /// Create a new linear scale
    pub fn new(domain: (f64, f64), range: (f64, f64)) -> Self {
        Self { domain, range }
    }

    /// Convenience constructor with domain and range from iterables
    pub fn from_extent(domain_min: f64, domain_max: f64, range_min: f64, range_max: f64) -> Self {
        Self::new((domain_min, domain_max), (range_min, range_max))
    }
}

impl Scale for LinearScale {
    fn map(&self, value: f64) -> f64 {
        let (d0, d1) = self.domain;
        let (r0, r1) = self.range;

        // Handle edge case: domain has zero width
        if (d1 - d0).abs() < f64::EPSILON {
            return r0;
        }

        // Linear interpolation: normalized * range_width + range_min
        let normalized = (value - d0) / (d1 - d0);
        r0 + normalized * (r1 - r0)
    }

    fn inverse(&self, mapped: f64) -> f64 {
        let (d0, d1) = self.domain;
        let (r0, r1) = self.range;

        // Handle edge case: range has zero width
        if (r1 - r0).abs() < f64::EPSILON {
            return d0;
        }

        // Inverse linear interpolation
        let normalized = (mapped - r0) / (r1 - r0);
        d0 + normalized * (d1 - d0)
    }

    fn domain(&self) -> (f64, f64) {
        self.domain
    }

    fn range(&self) -> (f64, f64) {
        self.range
    }
}

/// Logarithmic scale for data with large dynamic range
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LogScale {
    domain: (f64, f64),
    range: (f64, f64),
    base: f64,
}

impl LogScale {
    /// Create a new log scale with base 10
    pub fn new(domain: (f64, f64), range: (f64, f64)) -> Self {
        Self::with_base(domain, range, 10.0)
    }

    /// Create a new log scale with custom base
    pub fn with_base(domain: (f64, f64), range: (f64, f64), base: f64) -> Self {
        assert!(domain.0 > 0.0, "Log scale domain min must be > 0");
        assert!(domain.1 > 0.0, "Log scale domain max must be > 0");
        assert!(base > 0.0 && base != 1.0, "Log base must be > 0 and != 1");

        Self {
            domain,
            range,
            base,
        }
    }
}

impl Scale for LogScale {
    fn map(&self, value: f64) -> f64 {
        let (d0, d1) = self.domain;
        let (r0, r1) = self.range;

        if value <= 0.0 {
            return r0; // Clamp negative values to range min
        }

        // Log transformation: log(value) normalized within log(domain)
        let log_d0 = d0.log(self.base);
        let log_d1 = d1.log(self.base);
        let log_value = value.log(self.base);

        let normalized = (log_value - log_d0) / (log_d1 - log_d0);
        r0 + normalized * (r1 - r0)
    }

    fn inverse(&self, mapped: f64) -> f64 {
        let (d0, d1) = self.domain;
        let (r0, r1) = self.range;

        let normalized = (mapped - r0) / (r1 - r0);

        let log_d0 = d0.log(self.base);
        let log_d1 = d1.log(self.base);
        let log_value = log_d0 + normalized * (log_d1 - log_d0);

        self.base.powf(log_value)
    }

    fn domain(&self) -> (f64, f64) {
        self.domain
    }

    fn range(&self) -> (f64, f64) {
        self.range
    }
}

/// Time scale for temporal data (simplified version using f64 timestamps)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimeScale {
    /// Domain as Unix timestamps (seconds since epoch)
    domain: (f64, f64),
    range: (f64, f64),
}

impl TimeScale {
    /// Create a new time scale
    pub fn new(domain: (f64, f64), range: (f64, f64)) -> Self {
        Self { domain, range }
    }
}

impl Scale for TimeScale {
    fn map(&self, value: f64) -> f64 {
        // Time scale is just linear scale on timestamps
        let (d0, d1) = self.domain;
        let (r0, r1) = self.range;

        if (d1 - d0).abs() < f64::EPSILON {
            return r0;
        }

        let normalized = (value - d0) / (d1 - d0);
        r0 + normalized * (r1 - r0)
    }

    fn inverse(&self, mapped: f64) -> f64 {
        let (d0, d1) = self.domain;
        let (r0, r1) = self.range;

        if (r1 - r0).abs() < f64::EPSILON {
            return d0;
        }

        let normalized = (mapped - r0) / (r1 - r0);
        d0 + normalized * (d1 - d0)
    }

    fn domain(&self) -> (f64, f64) {
        self.domain
    }

    fn range(&self) -> (f64, f64) {
        self.range
    }
}

/// Band scale for categorical/ordinal data
///
/// Maps discrete categories to equal-width bands within the output range.
/// Used primarily for bar charts where each category gets a fixed-width band.
#[derive(Debug, Clone, PartialEq)]
pub struct BandScale {
    categories: Vec<String>,
    range: (f64, f64),
    /// Padding between bands as a fraction of band step (0.0 to 1.0)
    padding: f64,
}

impl BandScale {
    /// Create a new band scale
    ///
    /// `padding` is a fraction of the step between bands (0.0 = no gap, 0.5 = half gap)
    pub fn new(categories: Vec<String>, range: (f64, f64), padding: f64) -> Self {
        let padding = padding.clamp(0.0, 1.0);
        Self {
            categories,
            range,
            padding,
        }
    }

    /// The step size (band width + padding) between consecutive bands
    pub fn step(&self) -> f64 {
        if self.categories.is_empty() {
            return 0.0;
        }
        let range_width = (self.range.1 - self.range.0).abs();
        range_width / self.categories.len() as f64
    }

    /// The width of each band (step minus padding)
    pub fn band_width(&self) -> f64 {
        self.step() * (1.0 - self.padding)
    }

    /// Map a category name to its band start position
    ///
    /// Returns `None` if the category is not found
    pub fn map_category(&self, name: &str) -> Option<f64> {
        let idx = self.categories.iter().position(|c| c == name)?;
        Some(self.map_index(idx))
    }

    /// Map a category index to its band start position
    pub fn map_index(&self, idx: usize) -> f64 {
        let step = self.step();
        let pad_offset = step * self.padding / 2.0;
        let (r0, _) = self.range;
        r0 + idx as f64 * step + pad_offset
    }

    /// Map a category index to its band center position
    pub fn map_index_center(&self, idx: usize) -> f64 {
        self.map_index(idx) + self.band_width() / 2.0
    }

    /// Get the number of categories
    pub fn len(&self) -> usize {
        self.categories.len()
    }

    /// Check if scale has no categories
    pub fn is_empty(&self) -> bool {
        self.categories.is_empty()
    }

    /// Get all categories
    pub fn categories(&self) -> &[String] {
        &self.categories
    }

    /// Get the range
    pub fn range(&self) -> (f64, f64) {
        self.range
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f64 = 1e-10;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < EPSILON
    }

    #[test]
    fn test_linear_scale_basic() {
        let scale = LinearScale::new((0.0, 100.0), (0.0, 500.0));

        assert!(approx_eq(scale.map(0.0), 0.0));
        assert!(approx_eq(scale.map(50.0), 250.0));
        assert!(approx_eq(scale.map(100.0), 500.0));
    }

    #[test]
    fn test_linear_scale_inverse() {
        let scale = LinearScale::new((0.0, 100.0), (0.0, 500.0));

        assert!(approx_eq(scale.inverse(0.0), 0.0));
        assert!(approx_eq(scale.inverse(250.0), 50.0));
        assert!(approx_eq(scale.inverse(500.0), 100.0));
    }

    #[test]
    fn test_linear_scale_negative_domain() {
        let scale = LinearScale::new((-50.0, 50.0), (0.0, 100.0));

        assert!(approx_eq(scale.map(-50.0), 0.0));
        assert!(approx_eq(scale.map(0.0), 50.0));
        assert!(approx_eq(scale.map(50.0), 100.0));
    }

    #[test]
    fn test_linear_scale_zero_width_domain() {
        let scale = LinearScale::new((10.0, 10.0), (0.0, 100.0));
        // Should return range min for any value
        assert!(approx_eq(scale.map(5.0), 0.0));
        assert!(approx_eq(scale.map(10.0), 0.0));
        assert!(approx_eq(scale.map(15.0), 0.0));
    }

    #[test]
    fn test_log_scale_basic() {
        let scale = LogScale::new((1.0, 100.0), (0.0, 100.0));

        assert!(approx_eq(scale.map(1.0), 0.0));
        assert!(approx_eq(scale.map(10.0), 50.0));
        assert!(approx_eq(scale.map(100.0), 100.0));
    }

    #[test]
    fn test_log_scale_inverse() {
        let scale = LogScale::new((1.0, 100.0), (0.0, 100.0));

        assert!(approx_eq(scale.inverse(0.0), 1.0));
        assert!(approx_eq(scale.inverse(50.0), 10.0));
        assert!(approx_eq(scale.inverse(100.0), 100.0));
    }

    #[test]
    fn test_log_scale_custom_base() {
        let scale = LogScale::with_base((1.0, 8.0), (0.0, 300.0), 2.0);

        assert!(approx_eq(scale.map(1.0), 0.0));
        assert!(approx_eq(scale.map(2.0), 100.0));
        assert!(approx_eq(scale.map(4.0), 200.0));
        assert!(approx_eq(scale.map(8.0), 300.0));
    }

    #[test]
    fn test_log_scale_clamps_negative() {
        let scale = LogScale::new((1.0, 100.0), (0.0, 100.0));
        // Negative values should be clamped to range min
        assert!(approx_eq(scale.map(-10.0), 0.0));
        assert!(approx_eq(scale.map(0.0), 0.0));
    }

    #[test]
    fn test_time_scale_basic() {
        // Unix timestamps: 2024-01-01 to 2024-12-31 (simplified)
        let year_start = 1_704_067_200.0; // 2024-01-01 00:00:00 UTC
        let year_end = 1_735_689_600.0; // 2024-12-31 00:00:00 UTC

        let scale = TimeScale::new((year_start, year_end), (0.0, 1000.0));

        assert!(approx_eq(scale.map(year_start), 0.0));
        assert!(approx_eq(scale.map(year_end), 1000.0));

        // Mid-year should map to ~500
        let mid_year = (year_start + year_end) / 2.0;
        assert!(approx_eq(scale.map(mid_year), 500.0));
    }

    #[test]
    fn test_time_scale_inverse() {
        let year_start = 1_704_067_200.0;
        let year_end = 1_735_689_600.0;

        let scale = TimeScale::new((year_start, year_end), (0.0, 1000.0));

        assert!(approx_eq(scale.inverse(0.0), year_start));
        assert!(approx_eq(scale.inverse(1000.0), year_end));
    }

    // === BandScale tests ===

    fn cat(names: &[&str]) -> Vec<String> {
        names.iter().map(|s| (*s).to_string()).collect()
    }

    #[test]
    fn test_band_scale_basic() {
        let scale = BandScale::new(cat(&["A", "B", "C"]), (0.0, 300.0), 0.0);
        assert!(approx_eq(scale.step(), 100.0));
        assert!(approx_eq(scale.band_width(), 100.0));
        assert!(approx_eq(scale.map_index(0), 0.0));
        assert!(approx_eq(scale.map_index(1), 100.0));
        assert!(approx_eq(scale.map_index(2), 200.0));
    }

    #[test]
    fn test_band_scale_with_padding() {
        let scale = BandScale::new(cat(&["A", "B", "C"]), (0.0, 300.0), 0.2);
        // step = 100, padding_offset = 100 * 0.2 / 2 = 10
        // band_width = 100 * 0.8 = 80
        assert!(approx_eq(scale.step(), 100.0));
        assert!(approx_eq(scale.band_width(), 80.0));
        assert!(approx_eq(scale.map_index(0), 10.0));
        assert!(approx_eq(scale.map_index(1), 110.0));
        assert!(approx_eq(scale.map_index(2), 210.0));
    }

    #[test]
    fn test_band_scale_center() {
        let scale = BandScale::new(cat(&["X", "Y"]), (0.0, 200.0), 0.0);
        // step=100, band=100, center at 50 and 150
        assert!(approx_eq(scale.map_index_center(0), 50.0));
        assert!(approx_eq(scale.map_index_center(1), 150.0));
    }

    #[test]
    fn test_band_scale_map_category() {
        let scale = BandScale::new(cat(&["Apple", "Banana", "Cherry"]), (0.0, 600.0), 0.0);
        assert!(approx_eq(scale.map_category("Banana").unwrap(), 200.0));
        assert!(approx_eq(scale.map_category("Cherry").unwrap(), 400.0));
        assert!(scale.map_category("Missing").is_none());
    }

    #[test]
    fn test_band_scale_single_category() {
        let scale = BandScale::new(cat(&["Only"]), (0.0, 500.0), 0.0);
        assert!(approx_eq(scale.step(), 500.0));
        assert!(approx_eq(scale.band_width(), 500.0));
        assert!(approx_eq(scale.map_index(0), 0.0));
    }

    #[test]
    fn test_band_scale_empty() {
        let scale = BandScale::new(vec![], (0.0, 300.0), 0.0);
        assert!(scale.is_empty());
        assert_eq!(scale.len(), 0);
        assert!(approx_eq(scale.step(), 0.0));
        assert!(approx_eq(scale.band_width(), 0.0));
    }

    #[test]
    fn test_band_scale_padding_clamped() {
        // Padding > 1.0 should be clamped to 1.0
        let scale = BandScale::new(cat(&["A", "B"]), (0.0, 200.0), 2.0);
        assert!(approx_eq(scale.band_width(), 0.0)); // fully padded
    }

    #[test]
    fn test_band_scale_half_padding() {
        let scale = BandScale::new(cat(&["A", "B"]), (0.0, 200.0), 0.5);
        // step=100, band=50, pad_offset=25
        assert!(approx_eq(scale.step(), 100.0));
        assert!(approx_eq(scale.band_width(), 50.0));
        assert!(approx_eq(scale.map_index(0), 25.0));
        assert!(approx_eq(scale.map_index(1), 125.0));
    }
}

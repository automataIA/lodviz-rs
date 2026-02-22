/// Accessibility utilities for SVG charts (WCAG compliant)
///
/// Generates ARIA attributes and text alternatives for screen readers.
use super::data::DataPoint;
use super::mark::Mark;

/// Accessibility configuration for a chart
#[derive(Debug, Clone, Default)]
pub struct A11yConfig {
    /// Custom aria-label for the chart
    pub label: Option<String>,
    /// Custom long description
    pub description: Option<String>,
    /// Enable aria-live for streaming data updates
    pub live_region: bool,
}

/// Generated ARIA attributes for an SVG chart element
#[derive(Debug, Clone)]
pub struct A11yAttrs {
    /// The WAI-ARIA role (e.g., "img", "figure")
    pub role: &'static str,
    /// A short string identifying the element
    pub aria_label: String,
    /// A longer textual representation containing the summary of the chart
    pub description: String,
}

/// Generate a human-readable description of a chart
pub fn generate_chart_description(
    mark: Mark,
    data_len: usize,
    x_label: Option<&str>,
    y_label: Option<&str>,
) -> String {
    let mark_name = match mark {
        Mark::Line => "Line",
        Mark::Area => "Area",
        Mark::Bar => "Bar",
        Mark::Point => "Scatter",
        Mark::Circle => "Bubble",
    };

    let mut desc = format!("{mark_name} chart with {data_len} data points.");

    if let Some(x) = x_label {
        desc.push_str(&format!(" X axis: {x}."));
    }
    if let Some(y) = y_label {
        desc.push_str(&format!(" Y axis: {y}."));
    }

    desc
}

/// Generate a text summary of data range for screen readers
pub fn summarize_data_range(data: &[DataPoint]) -> String {
    if data.is_empty() {
        return "No data points.".to_string();
    }

    let x_min = data.iter().map(|p| p.x).fold(f64::INFINITY, f64::min);
    let x_max = data.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max);
    let y_min = data.iter().map(|p| p.y).fold(f64::INFINITY, f64::min);
    let y_max = data.iter().map(|p| p.y).fold(f64::NEG_INFINITY, f64::max);

    format!("X range: {x_min:.2} to {x_max:.2}. Y range: {y_min:.2} to {y_max:.2}.")
}

/// Describe a single data point for screen reader announcement
pub fn describe_point(point: &DataPoint, index: usize, total: usize) -> String {
    format!(
        "Point {} of {}: x = {:.2}, y = {:.2}",
        index + 1,
        total,
        point.x,
        point.y
    )
}

/// Generate description for a named series within a multi-series chart
pub fn generate_series_description(name: &str, data: &[DataPoint]) -> String {
    if data.is_empty() {
        return format!("Series \"{name}\" with no data points.");
    }
    let range = summarize_data_range(data);
    format!("Series \"{name}\" with {} data points. {range}", data.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_chart_description_line() {
        let desc = generate_chart_description(Mark::Line, 100, Some("time"), Some("value"));
        assert!(desc.contains("Line chart"));
        assert!(desc.contains("100 data points"));
        assert!(desc.contains("X axis: time"));
        assert!(desc.contains("Y axis: value"));
    }

    #[test]
    fn test_generate_chart_description_scatter_no_labels() {
        let desc = generate_chart_description(Mark::Point, 50, None, None);
        assert!(desc.contains("Scatter chart"));
        assert!(desc.contains("50 data points"));
        assert!(!desc.contains("X axis"));
    }

    #[test]
    fn test_summarize_data_range() {
        let data = vec![
            DataPoint::new(0.0, 10.0),
            DataPoint::new(5.0, 20.0),
            DataPoint::new(10.0, 5.0),
        ];
        let summary = summarize_data_range(&data);
        assert!(summary.contains("X range: 0.00 to 10.00"));
        assert!(summary.contains("Y range: 5.00 to 20.00"));
    }

    #[test]
    fn test_summarize_data_range_empty() {
        let summary = summarize_data_range(&[]);
        assert_eq!(summary, "No data points.");
    }

    #[test]
    fn test_describe_point() {
        let point = DataPoint::new(std::f64::consts::PI, 2.72);
        let desc = describe_point(&point, 2, 10);
        assert!(desc.contains("Point 3 of 10"));
        assert!(desc.contains("x = 3.14"));
        assert!(desc.contains("y = 2.72"));
    }
}

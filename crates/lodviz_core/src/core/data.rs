/// Data structures for representing visualization data
///
/// A single data point with x and y coordinates
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DataPoint {
    /// The x-coordinate value
    pub x: f64,
    /// The y-coordinate value
    pub y: f64,
}

impl DataPoint {
    /// Create a new data point
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

/// A series of data points with a name
#[derive(Debug, Clone)]
pub struct Series<T> {
    /// The name of the series, often used in legends
    pub name: String,
    /// The collection of underlying data points
    pub data: Vec<T>,
    /// Whether the series is currently visible on the chart
    pub visible: bool,
}

impl<T> Series<T> {
    /// Create a new series
    pub fn new(name: impl Into<String>, data: Vec<T>) -> Self {
        Self {
            name: name.into(),
            data,
            visible: true,
        }
    }
}

/// A dataset containing multiple series
#[derive(Debug, Clone)]
pub struct Dataset {
    /// The collection of series forming this dataset
    pub series: Vec<Series<DataPoint>>,
}

impl Dataset {
    /// Create a new empty dataset
    pub fn new() -> Self {
        Self { series: Vec::new() }
    }

    /// Add a series to the dataset
    pub fn add_series(&mut self, series: Series<DataPoint>) {
        self.series.push(series);
    }

    /// Create a dataset with a single series
    pub fn from_series(series: Series<DataPoint>) -> Self {
        Self {
            series: vec![series],
        }
    }
}

impl Default for Dataset {
    fn default() -> Self {
        Self::new()
    }
}

/// OHLC (Open, High, Low, Close) price bar for candlestick charts
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OhlcBar {
    /// X position, typically a Unix timestamp or sequential index
    pub timestamp: f64,
    /// The opening price of the period
    pub open: f64,
    /// The highest price reached during the period
    pub high: f64,
    /// The lowest price reached during the period
    pub low: f64,
    /// The closing price of the period
    pub close: f64,
}

impl OhlcBar {
    /// Create a new OHLC bar
    pub fn new(timestamp: f64, open: f64, high: f64, low: f64, close: f64) -> Self {
        Self {
            timestamp,
            open,
            high,
            low,
            close,
        }
    }

    /// Returns `true` when close ≥ open (bullish / green candle)
    pub fn is_bullish(&self) -> bool {
        self.close >= self.open
    }
}

/// Category of a waterfall bar
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaterfallKind {
    /// Opening value (baseline starts at zero)
    Start,
    /// Incremental delta — positive (up) or negative (down)
    Delta,
    /// Running total shown from zero
    Total,
}

/// A single bar in a waterfall chart
#[derive(Debug, Clone)]
pub struct WaterfallBar {
    /// The textual label of the category/step
    pub label: String,
    /// The numerical value (can be absolute or delta)
    pub value: f64,
    /// The type characterizing how this bar behaves
    pub kind: WaterfallKind,
}

impl WaterfallBar {
    /// Delta (incremental) bar
    pub fn delta(label: impl Into<String>, value: f64) -> Self {
        Self {
            label: label.into(),
            value,
            kind: WaterfallKind::Delta,
        }
    }

    /// Start bar — initial baseline
    pub fn start(label: impl Into<String>, value: f64) -> Self {
        Self {
            label: label.into(),
            value,
            kind: WaterfallKind::Start,
        }
    }

    /// Total bar — cumulative sum shown from zero
    pub fn total(label: impl Into<String>, value: f64) -> Self {
        Self {
            label: label.into(),
            value,
            kind: WaterfallKind::Total,
        }
    }
}

/// A category-based dataset for bar charts
///
/// Each series provides one value per category.
#[derive(Debug, Clone, Default)]
pub struct BarDataset {
    /// The list of category labels on the primary axis
    pub categories: Vec<String>,
    /// The collection of data series mapping to the categories
    pub series: Vec<BarSeries>,
}

impl BarDataset {
    /// Create a new bar dataset with the given categories
    pub fn new(categories: Vec<String>) -> Self {
        Self {
            categories,
            series: Vec::new(),
        }
    }

    /// Add a named series with one value per category
    pub fn add_series(&mut self, name: impl Into<String>, values: Vec<f64>) {
        self.series.push(BarSeries {
            name: name.into(),
            values,
        });
    }
}

/// A single named series for a bar chart
#[derive(Debug, Clone)]
pub struct BarSeries {
    /// The identifier name for this data series
    pub name: String,
    /// The actual numerical values, usually 1:1 with categories length
    pub values: Vec<f64>,
}

/// Type of data for encoding channels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataType {
    /// Continuous numerical data
    Quantitative,
    /// Date/time data
    Temporal,
    /// Categorical data (unordered)
    Nominal,
    /// Categorical data (ordered)
    Ordinal,
}

// ---------------------------------------------------------------------------
// New data types for Heatmap, Strip, Sankey, Chord charts
// ---------------------------------------------------------------------------

/// A 2-D grid of values (rows × columns), used by HeatmapChart and ContourChart
#[derive(Debug, Clone, Default)]
pub struct GridData {
    /// Row-major matrix of values: `values[row][col]`
    pub values: Vec<Vec<f64>>,
    /// Optional labels for each row (Y axis)
    pub row_labels: Option<Vec<String>>,
    /// Optional labels for each column (X axis)
    pub col_labels: Option<Vec<String>>,
}

impl GridData {
    /// Minimum value in the grid, or 0.0 if empty
    pub fn min(&self) -> f64 {
        self.values
            .iter()
            .flatten()
            .cloned()
            .fold(f64::INFINITY, f64::min)
            .clamp(0.0, f64::MAX)
    }

    /// Maximum value in the grid, or 1.0 if empty
    pub fn max(&self) -> f64 {
        self.values
            .iter()
            .flatten()
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max)
    }
}

/// A named group of values for a StripChart
#[derive(Debug, Clone)]
pub struct StripGroup {
    /// Display name for this group (shown on the categorical axis)
    pub name: String,
    /// The individual data values within this group
    pub values: Vec<f64>,
}

/// A node in a Sankey diagram
#[derive(Debug, Clone)]
pub struct SankeyNode {
    /// Display label for this node
    pub label: String,
    /// Optional override color (hex string). If None, uses the palette.
    pub color: Option<String>,
}

/// A directional flow link between two Sankey nodes
#[derive(Debug, Clone)]
pub struct SankeyLink {
    /// Index into `SankeyData::nodes` for the source
    pub source: usize,
    /// Index into `SankeyData::nodes` for the target
    pub target: usize,
    /// Flow magnitude (proportional to ribbon width)
    pub value: f64,
    /// Optional override color (hex string). If None, uses source node color.
    pub color: Option<String>,
}

/// Complete data for a Sankey flow diagram
#[derive(Debug, Clone, Default)]
pub struct SankeyData {
    /// All nodes in the diagram
    pub nodes: Vec<SankeyNode>,
    /// All directed links between nodes
    pub links: Vec<SankeyLink>,
}

/// Complete data for a Chord diagram
#[derive(Debug, Clone, Default)]
pub struct ChordData {
    /// Square matrix where `matrix[i][j]` = flow from group i to group j
    pub matrix: Vec<Vec<f64>>,
    /// Labels for each group (one per row/column)
    pub labels: Vec<String>,
    /// Optional per-group colors (hex strings). If None, uses the palette.
    pub colors: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_point_creation() {
        let point = DataPoint::new(1.0, 2.0);
        assert_eq!(point.x, 1.0);
        assert_eq!(point.y, 2.0);
    }

    #[test]
    fn test_series_creation() {
        let data = vec![DataPoint::new(1.0, 2.0), DataPoint::new(3.0, 4.0)];
        let series = Series::new("test", data.clone());
        assert_eq!(series.name, "test");
        assert_eq!(series.data.len(), 2);
    }

    #[test]
    fn test_dataset_creation() {
        let mut dataset = Dataset::new();
        assert_eq!(dataset.series.len(), 0);

        let series = Series::new("s1", vec![DataPoint::new(1.0, 2.0)]);
        dataset.add_series(series);
        assert_eq!(dataset.series.len(), 1);
    }

    #[test]
    fn test_dataset_from_series() {
        let series = Series::new("s1", vec![DataPoint::new(1.0, 2.0)]);
        let dataset = Dataset::from_series(series);
        assert_eq!(dataset.series.len(), 1);
        assert_eq!(dataset.series[0].name, "s1");
    }
}

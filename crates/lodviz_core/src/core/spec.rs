/// Typestate builder for ChartSpec
///
/// Uses zero-sized type markers to enforce required fields at compile time.
/// `build()` is only available when all required fields (data, mark, x) are set.
use super::data::{BarDataset, DataPoint, Dataset, Series};
use super::encoding::Field;
use super::field_value::DataTable;
use super::mark::Mark;
use super::theme::{ChartConfig, GridStyle};

// --- Typestate markers ---

/// Marker: required field not yet set
pub struct Missing;

/// Marker: required field has been set
pub struct Set<T>(pub T);

// --- ChartData ---

/// Unified data source for a chart spec.
///
/// Allows passing pre-built datasets, categorical bar data,
/// or a raw `DataTable` that gets converted at render time
/// based on the `Mark` type and `Encoding` fields in `ChartSpec`.
#[derive(Debug, Clone)]
pub enum ChartData {
    /// Continuous time-series or scatter data (`Dataset`)
    TimeSeries(Dataset),
    /// Category-based data for bar charts (`BarDataset`)
    Categorical(BarDataset),
    /// Raw tidy table — converted lazily by the chart facade
    Table(DataTable),
}

impl ChartData {
    /// Attempt to get a reference to the inner `Dataset`
    pub fn as_dataset(&self) -> Option<&Dataset> {
        match self {
            Self::TimeSeries(ds) => Some(ds),
            _ => None,
        }
    }

    /// Attempt to get a reference to the inner `BarDataset`
    pub fn as_bar_dataset(&self) -> Option<&BarDataset> {
        match self {
            Self::Categorical(bd) => Some(bd),
            _ => None,
        }
    }

    /// Attempt to get a reference to the inner `DataTable`
    pub fn as_table(&self) -> Option<&DataTable> {
        match self {
            Self::Table(t) => Some(t),
            _ => None,
        }
    }
}

// --- Final spec ---

/// Immutable chart specification with all fields resolved
#[derive(Debug, Clone)]
pub struct ChartSpec {
    /// The data source driving the chart
    pub data: ChartData,
    /// The visual mark (Line, Bar, Point, etc.)
    pub mark: Mark,
    /// The field encoding for the primary X dimension
    pub x: Field,
    /// The optional field encoding for the Y dimension
    pub y: Option<Field>,
    /// The optional field encoding for the color channel
    pub color: Option<Field>,
    /// The optional field encoding for the size channel
    pub size: Option<Field>,
    /// Global layout and styling configuration
    pub config: ChartConfig,
}

impl ChartSpec {
    /// Start building a new chart spec
    pub fn builder() -> ChartSpecBuilder<Missing, Missing, Missing> {
        ChartSpecBuilder {
            data: Missing,
            mark: Missing,
            x: Missing,
            y: None,
            color: None,
            size: None,
            config: ChartConfig::default(),
        }
    }
}

// --- Builder ---

/// Typestate builder — generic parameters track which required fields are set
pub struct ChartSpecBuilder<D, M, X> {
    data: D,
    mark: M,
    x: X,
    y: Option<Field>,
    color: Option<Field>,
    size: Option<Field>,
    config: ChartConfig,
}

// Required setters: each consumes self and changes one type parameter

impl<M, X> ChartSpecBuilder<Missing, M, X> {
    /// Set the dataset directly (required)
    pub fn data(self, dataset: Dataset) -> ChartSpecBuilder<Set<ChartData>, M, X> {
        ChartSpecBuilder {
            data: Set(ChartData::TimeSeries(dataset)),
            mark: self.mark,
            x: self.x,
            y: self.y,
            color: self.color,
            size: self.size,
            config: self.config,
        }
    }

    /// Convenience: wrap a `Vec<DataPoint>` into a single-series `Dataset`
    pub fn data_points(self, points: Vec<DataPoint>) -> ChartSpecBuilder<Set<ChartData>, M, X> {
        let series = Series::new("default", points);
        let dataset = Dataset::from_series(series);
        self.data(dataset)
    }

    /// Set a raw `BarDataset` for categorical bar charts (required)
    pub fn bar_data(self, bar_dataset: BarDataset) -> ChartSpecBuilder<Set<ChartData>, M, X> {
        ChartSpecBuilder {
            data: Set(ChartData::Categorical(bar_dataset)),
            mark: self.mark,
            x: self.x,
            y: self.y,
            color: self.color,
            size: self.size,
            config: self.config,
        }
    }

    /// Set a `DataTable` as the data source — converted lazily at render time.
    ///
    /// Call `.x()`, `.y()`, and optionally `.color()` to specify which columns
    /// to use. The `SmartChart` facade will convert to the appropriate dataset
    /// type based on the `Mark`.
    pub fn from_table(self, table: DataTable) -> ChartSpecBuilder<Set<ChartData>, M, X> {
        ChartSpecBuilder {
            data: Set(ChartData::Table(table)),
            mark: self.mark,
            x: self.x,
            y: self.y,
            color: self.color,
            size: self.size,
            config: self.config,
        }
    }
}

impl<D, X> ChartSpecBuilder<D, Missing, X> {
    /// Set the mark type (required)
    pub fn mark(self, mark: Mark) -> ChartSpecBuilder<D, Set<Mark>, X> {
        ChartSpecBuilder {
            data: self.data,
            mark: Set(mark),
            x: self.x,
            y: self.y,
            color: self.color,
            size: self.size,
            config: self.config,
        }
    }
}

impl<D, M> ChartSpecBuilder<D, M, Missing> {
    /// Set the x encoding field (required)
    pub fn x(self, field: Field) -> ChartSpecBuilder<D, M, Set<Field>> {
        ChartSpecBuilder {
            data: self.data,
            mark: self.mark,
            x: Set(field),
            y: self.y,
            color: self.color,
            size: self.size,
            config: self.config,
        }
    }
}

// Optional setters: available in any state

impl<D, M, X> ChartSpecBuilder<D, M, X> {
    /// Set the y encoding field (optional)
    pub fn y(mut self, field: Field) -> Self {
        self.y = Some(field);
        self
    }

    /// Set the color encoding field (optional)
    pub fn color(mut self, field: Field) -> Self {
        self.color = Some(field);
        self
    }

    /// Set the size encoding field (optional)
    pub fn size(mut self, field: Field) -> Self {
        self.size = Some(field);
        self
    }

    /// Set the chart title (optional)
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.config.title = Some(title.into());
        self
    }

    /// Toggle grid lines (optional)
    pub fn grid(mut self, show: bool) -> Self {
        self.config.grid = Some(GridStyle {
            show_x: show,
            show_y: show,
            ..GridStyle::default()
        });
        self
    }

    /// Set full chart config (optional)
    pub fn config(mut self, config: ChartConfig) -> Self {
        self.config = config;
        self
    }
}

// build() only when all required fields are Set

impl ChartSpecBuilder<Set<ChartData>, Set<Mark>, Set<Field>> {
    /// Build the final immutable `ChartSpec`
    pub fn build(self) -> ChartSpec {
        ChartSpec {
            data: self.data.0,
            mark: self.mark.0,
            x: self.x.0,
            y: self.y,
            color: self.color,
            size: self.size,
            config: self.config,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::data::DataType;

    fn sample_points() -> Vec<DataPoint> {
        vec![
            DataPoint::new(0.0, 1.0),
            DataPoint::new(1.0, 3.0),
            DataPoint::new(2.0, 2.0),
        ]
    }

    #[test]
    fn test_builder_all_required() {
        let spec = ChartSpec::builder()
            .data_points(sample_points())
            .mark(Mark::Line)
            .x(Field::temporal("time"))
            .build();

        assert_eq!(spec.mark, Mark::Line);
        assert_eq!(spec.x.name, "time");
        assert_eq!(spec.x.data_type, DataType::Temporal);
        let ds = spec.data.as_dataset().expect("expected TimeSeries");
        assert_eq!(ds.series.len(), 1);
        assert_eq!(ds.series[0].data.len(), 3);
        assert!(spec.y.is_none());
        assert!(spec.color.is_none());
        assert!(spec.size.is_none());
    }

    #[test]
    fn test_builder_with_optionals() {
        let spec = ChartSpec::builder()
            .data_points(sample_points())
            .mark(Mark::Point)
            .x(Field::quantitative("x"))
            .y(Field::quantitative("y"))
            .color(Field::nominal("category"))
            .size(Field::quantitative("magnitude"))
            .title("My Chart")
            .grid(true)
            .build();

        assert_eq!(spec.y.as_ref().unwrap().name, "y");
        assert_eq!(spec.color.as_ref().unwrap().name, "category");
        assert_eq!(spec.size.as_ref().unwrap().name, "magnitude");
        assert_eq!(spec.config.title.as_deref(), Some("My Chart"));
        assert!(spec.config.grid.as_ref().unwrap().show_x);
    }

    #[test]
    fn test_builder_any_order() {
        // mark -> x -> data
        let spec1 = ChartSpec::builder()
            .mark(Mark::Bar)
            .x(Field::nominal("category"))
            .data_points(sample_points())
            .build();

        // x -> data -> mark
        let spec2 = ChartSpec::builder()
            .x(Field::nominal("category"))
            .data_points(sample_points())
            .mark(Mark::Bar)
            .build();

        assert_eq!(spec1.mark, spec2.mark);
        assert_eq!(spec1.x.name, spec2.x.name);
    }

    #[test]
    fn test_data_points_convenience() {
        let points = sample_points();
        let spec = ChartSpec::builder()
            .data_points(points.clone())
            .mark(Mark::Line)
            .x(Field::quantitative("x"))
            .build();

        let ds = spec.data.as_dataset().unwrap();
        assert_eq!(ds.series.len(), 1);
        assert_eq!(ds.series[0].name, "default");
        assert_eq!(ds.series[0].data.len(), points.len());
    }

    #[test]
    fn test_data_dataset_direct() {
        let mut dataset = Dataset::new();
        dataset.add_series(Series::new("s1", sample_points()));
        dataset.add_series(Series::new("s2", sample_points()));

        let spec = ChartSpec::builder()
            .data(dataset)
            .mark(Mark::Line)
            .x(Field::temporal("time"))
            .build();

        assert_eq!(spec.data.as_dataset().unwrap().series.len(), 2);
    }

    #[test]
    fn test_default_config() {
        let spec = ChartSpec::builder()
            .data_points(sample_points())
            .mark(Mark::Line)
            .x(Field::quantitative("x"))
            .build();

        assert!(spec.config.title.is_none());
        assert!(spec.config.grid.is_none());
    }

    #[test]
    fn test_from_table_stores_table() {
        use crate::core::field_value::{DataTable, FieldValue};
        use std::collections::HashMap;

        let mut table = DataTable::default();
        let mut row = HashMap::new();
        row.insert("x".into(), FieldValue::Numeric(1.0));
        row.insert("y".into(), FieldValue::Numeric(2.0));
        table.push(row);

        let spec = ChartSpec::builder()
            .from_table(table)
            .mark(Mark::Line)
            .x(Field::temporal("x"))
            .y(Field::quantitative("y"))
            .build();

        assert!(spec.data.as_table().is_some());
        assert_eq!(spec.data.as_table().unwrap().len(), 1);
    }

    #[test]
    fn test_bar_data_categorical() {
        let mut bd = BarDataset::new(vec!["Q1".into(), "Q2".into()]);
        bd.add_series("Revenue", vec![100.0, 150.0]);

        let spec = ChartSpec::builder()
            .bar_data(bd)
            .mark(Mark::Bar)
            .x(Field::nominal("period"))
            .build();

        assert!(spec.data.as_bar_dataset().is_some());
        assert_eq!(spec.data.as_bar_dataset().unwrap().series.len(), 1);
    }
}

/// Tidy data layer: typed field values, rows, and convertible table
///
/// `DataTable` is the entry point for the "raw data → chart" pipeline.
/// Users pass business data as rows of named columns; the table converts
/// to `Dataset` or `BarDataset` using an `Encoding` specification.
use std::collections::HashMap;

use crate::core::data::{
    BarDataset, ChordData, DataPoint, Dataset, GridData, SankeyData, SankeyLink, SankeyNode,
    Series, StripGroup,
};
use crate::core::encoding::Encoding;

// --- FieldValue ---

/// A single typed value in a data column
#[derive(Debug, Clone, PartialEq)]
pub enum FieldValue {
    /// Continuous numerical value (f64)
    Numeric(f64),
    /// String / categorical value
    Text(String),
    /// Unix timestamp in milliseconds
    Timestamp(f64),
    /// Boolean flag
    Bool(bool),
    /// Missing / null value
    Null,
}

impl FieldValue {
    /// Try to cast to f64 (Numeric, Timestamp, Bool → 0.0/1.0)
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Numeric(v) => Some(*v),
            Self::Timestamp(v) => Some(*v),
            Self::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
            Self::Text(_) | Self::Null => None,
        }
    }

    /// Try to cast to &str (Text only)
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::Text(s) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Try to cast to Unix timestamp ms (Timestamp or Numeric)
    pub fn as_timestamp(&self) -> Option<f64> {
        match self {
            Self::Timestamp(v) | Self::Numeric(v) => Some(*v),
            _ => None,
        }
    }

    /// Returns true if this value is Null
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }
}

impl From<f64> for FieldValue {
    fn from(v: f64) -> Self {
        Self::Numeric(v)
    }
}

impl From<i64> for FieldValue {
    fn from(v: i64) -> Self {
        Self::Numeric(v as f64)
    }
}

impl From<i32> for FieldValue {
    fn from(v: i32) -> Self {
        Self::Numeric(f64::from(v))
    }
}

impl From<String> for FieldValue {
    fn from(s: String) -> Self {
        Self::Text(s)
    }
}

impl From<&str> for FieldValue {
    fn from(s: &str) -> Self {
        Self::Text(s.to_owned())
    }
}

impl From<bool> for FieldValue {
    fn from(b: bool) -> Self {
        Self::Bool(b)
    }
}

// --- DataRow ---

/// A single row of named column → typed value mappings.
///
/// Uses `HashMap` for O(1) column lookup with no extra dependencies.
pub type DataRow = HashMap<String, FieldValue>;

// --- DataTable ---

/// A tidy table of heterogeneous rows, convertible to chart datasets.
///
/// Each row is a `DataRow` (`HashMap<String, FieldValue>`).
/// Conversion methods (`to_dataset`, `to_bar_dataset`) use an `Encoding`
/// to select which columns map to x, y, and color channels.
#[derive(Debug, Clone, Default)]
pub struct DataTable {
    rows: Vec<DataRow>,
}

impl DataTable {
    /// Create a new table from a pre-built list of rows
    pub fn new(rows: Vec<DataRow>) -> Self {
        Self { rows }
    }

    /// Alias for `new` — matches the target developer-experience API
    pub fn from_rows(rows: Vec<DataRow>) -> Self {
        Self::new(rows)
    }

    /// Append a row to the table
    pub fn push(&mut self, row: DataRow) {
        self.rows.push(row);
    }

    /// Number of rows
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// True when the table has no rows
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Read-only slice of all rows
    pub fn rows(&self) -> &[DataRow] {
        &self.rows
    }

    // --- Column extraction helpers ---

    /// Extract all non-null numeric values from a column (skips Null rows)
    pub fn extract_numeric(&self, col: &str) -> Vec<f64> {
        self.rows
            .iter()
            .filter_map(|row| row.get(col)?.as_f64())
            .collect()
    }

    /// Extract all text values from a column (skips non-Text rows)
    pub fn extract_text(&self, col: &str) -> Vec<String> {
        self.rows
            .iter()
            .filter_map(|row| row.get(col)?.as_str().map(ToOwned::to_owned))
            .collect()
    }

    // --- Grouping ---

    /// Split the table into sub-tables by the distinct values of `col`.
    ///
    /// Returns groups in first-occurrence order, as `(group_key, DataTable)`.
    /// Used for `color` encoding → multi-series generation.
    pub fn group_by(&self, col: &str) -> Vec<(String, Self)> {
        let mut order: Vec<String> = Vec::new();
        let mut groups: HashMap<String, Vec<DataRow>> = HashMap::new();

        for row in &self.rows {
            let key = match row.get(col) {
                Some(FieldValue::Text(s)) => s.clone(),
                Some(FieldValue::Numeric(v)) => v.to_string(),
                Some(FieldValue::Timestamp(v)) => v.to_string(),
                Some(FieldValue::Bool(b)) => b.to_string(),
                Some(FieldValue::Null) | None => String::from("__null__"),
            };
            if !groups.contains_key(&key) {
                order.push(key.clone());
            }
            groups.entry(key).or_default().push(row.clone());
        }

        order
            .into_iter()
            .map(|key| {
                let rows = groups.remove(&key).unwrap_or_default();
                (key, Self::new(rows))
            })
            .collect()
    }

    // --- Dataset conversion ---

    /// Convert to `Dataset` for line / scatter / area charts.
    ///
    /// - `encoding.x` → x column (numeric / timestamp)
    /// - `encoding.y` → y column (numeric)
    /// - `encoding.color` → if set, groups rows into multiple `Series`
    ///
    /// Rows where x or y are missing / non-numeric are silently skipped.
    pub fn to_dataset(&self, encoding: &Encoding) -> Dataset {
        let color_col = encoding.color.as_ref().map(|f| f.name.as_str());

        if let Some(col) = color_col {
            let groups = self.group_by(col);
            let mut dataset = Dataset::new();
            for (name, sub) in groups {
                let points = sub.extract_xy(&encoding.x.name, &encoding.y.name);
                dataset.add_series(Series::new(name, points));
            }
            dataset
        } else {
            let points = self.extract_xy(&encoding.x.name, &encoding.y.name);
            Dataset::from_series(Series::new("default", points))
        }
    }

    /// Convert to `BarDataset` for bar charts.
    ///
    /// - `encoding.x` → category column (text labels become bar categories)
    /// - `encoding.y` → value column (numeric bar heights)
    /// - `encoding.color` → if set, groups rows into multiple `BarSeries`
    ///
    /// When `color` is absent, produces a single series named `"default"`.
    /// Rows where the value is missing / non-numeric are recorded as `0.0`.
    pub fn to_bar_dataset(&self, encoding: &Encoding) -> BarDataset {
        let cat_col = &encoding.x.name;
        let val_col = &encoding.y.name;
        let series_col = encoding.color.as_ref().map(|f| f.name.as_str());

        // Collect ordered unique categories
        let categories: Vec<String> = {
            let mut seen = std::collections::HashSet::new();
            let mut ordered = Vec::new();
            for row in &self.rows {
                if let Some(FieldValue::Text(s)) = row.get(cat_col.as_str()) {
                    if seen.insert(s.clone()) {
                        ordered.push(s.clone());
                    }
                }
            }
            ordered
        };

        let mut bar_dataset = BarDataset::new(categories.clone());

        if let Some(scol) = series_col {
            // Multi-series: one BarSeries per distinct color-field value
            let groups = self.group_by(scol);
            for (name, sub) in groups {
                let values: Vec<f64> = categories
                    .iter()
                    .map(|cat| {
                        sub.rows
                            .iter()
                            .find(|row| {
                                row.get(cat_col.as_str())
                                    .and_then(FieldValue::as_str)
                                    .map(|s| s == cat.as_str())
                                    .unwrap_or(false)
                            })
                            .and_then(|row| row.get(val_col.as_str())?.as_f64())
                            .unwrap_or(0.0)
                    })
                    .collect();
                bar_dataset.add_series(name, values);
            }
        } else {
            // Single series
            let values: Vec<f64> = categories
                .iter()
                .map(|cat| {
                    self.rows
                        .iter()
                        .find(|row| {
                            row.get(cat_col.as_str())
                                .and_then(FieldValue::as_str)
                                .map(|s| s == cat.as_str())
                                .unwrap_or(false)
                        })
                        .and_then(|row| row.get(val_col.as_str())?.as_f64())
                        .unwrap_or(0.0)
                })
                .collect();
            bar_dataset.add_series("default", values);
        }

        bar_dataset
    }

    // --- New chart type conversions ---

    /// Long/tidy table → `Vec<StripGroup>` for `StripChart`.
    ///
    /// Groups rows by the distinct values of `group_col`; all numeric values
    /// in `value_col` within a group are collected into one `StripGroup`.
    /// Rows with missing values are silently skipped.
    ///
    /// ```text
    /// group  | value
    /// -------+------
    /// Control| 2.1
    /// Control| 1.9
    /// Group A| 4.5
    /// ```
    pub fn to_strip_groups(&self, group_col: &str, value_col: &str) -> Vec<StripGroup> {
        self.group_by(group_col)
            .into_iter()
            .map(|(name, sub)| StripGroup {
                name,
                values: sub.extract_numeric(value_col),
            })
            .collect()
    }

    /// Edge-list table → `SankeyData` for `SankeyChart`.
    ///
    /// Each row is a directed link. Node labels are inferred from the
    /// source and target columns in first-seen order — no separate node table
    /// is required. The optional `color_col` sets the link color (e.g. `"#e15759"`).
    ///
    /// ```text
    /// source      | target      | value
    /// ------------+-------------+------
    /// Coal        | Electricity | 40
    /// Coal        | Heat        | 20
    /// Gas         | Electricity | 30
    /// ```
    pub fn to_sankey(
        &self,
        source_col: &str,
        target_col: &str,
        value_col: &str,
        color_col: Option<&str>,
    ) -> SankeyData {
        let mut label_index: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        let mut nodes: Vec<SankeyNode> = Vec::new();
        let mut links: Vec<SankeyLink> = Vec::new();

        for row in &self.rows {
            let Some(src_label) = row
                .get(source_col)
                .and_then(FieldValue::as_str)
                .map(str::to_owned)
            else {
                continue;
            };
            let Some(dst_label) = row
                .get(target_col)
                .and_then(FieldValue::as_str)
                .map(str::to_owned)
            else {
                continue;
            };
            let Some(value) = row.get(value_col).and_then(FieldValue::as_f64) else {
                continue;
            };

            let color = color_col
                .and_then(|col| row.get(col))
                .and_then(FieldValue::as_str)
                .map(str::to_owned);

            let source = *label_index.entry(src_label.clone()).or_insert_with(|| {
                let idx = nodes.len();
                nodes.push(SankeyNode {
                    label: src_label,
                    color: None,
                });
                idx
            });
            let target = *label_index.entry(dst_label.clone()).or_insert_with(|| {
                let idx = nodes.len();
                nodes.push(SankeyNode {
                    label: dst_label,
                    color: None,
                });
                idx
            });

            links.push(SankeyLink {
                source,
                target,
                value,
                color,
            });
        }

        SankeyData { nodes, links }
    }

    /// Wide square-matrix table → `ChordData` for `ChordChart`.
    ///
    /// `label_col` contains group names; `value_cols` are the flow columns
    /// in the same order as the groups (i.e. a symmetric or asymmetric NxN matrix).
    ///
    /// ```text
    /// label    | Europe | Americas | Asia
    /// ---------+--------+----------+-----
    /// Europe   |      0 |       12 |    8
    /// Americas |      7 |        0 |   15
    /// Asia     |      6 |       11 |    0
    /// ```
    pub fn to_chord_matrix(&self, label_col: &str, value_cols: &[&str]) -> ChordData {
        let labels: Vec<String> = self
            .rows
            .iter()
            .filter_map(|row| row.get(label_col)?.as_str().map(str::to_owned))
            .collect();

        let matrix: Vec<Vec<f64>> = self
            .rows
            .iter()
            .map(|row| {
                value_cols
                    .iter()
                    .map(|col| row.get(*col).and_then(FieldValue::as_f64).unwrap_or(0.0))
                    .collect()
            })
            .collect();

        ChordData {
            matrix,
            labels,
            colors: None,
        }
    }

    /// Wide-format table → `GridData` for `HeatmapChart` / `ContourChart`.
    ///
    /// Each row of the table becomes one row of the grid. `value_cols` are
    /// the column names whose numeric values fill the cells (they also become
    /// column labels). The optional `label_col` provides row labels.
    ///
    /// ```text
    /// name      | Jan  | Feb  | Mar
    /// ----------+------+------+-----
    /// Product A | 10.5 |  8.2 | 12.1
    /// Product B |  6.3 |  9.0 |  7.4
    /// ```
    pub fn to_grid_wide(&self, label_col: Option<&str>, value_cols: &[&str]) -> GridData {
        let row_labels = label_col.map(|col| {
            self.rows
                .iter()
                .filter_map(|row| row.get(col)?.as_str().map(str::to_owned))
                .collect()
        });

        let values: Vec<Vec<f64>> = self
            .rows
            .iter()
            .map(|row| {
                value_cols
                    .iter()
                    .map(|col| row.get(*col).and_then(FieldValue::as_f64).unwrap_or(0.0))
                    .collect()
            })
            .collect();

        let col_labels = Some(value_cols.iter().map(|s| (*s).to_owned()).collect());

        GridData {
            values,
            row_labels,
            col_labels,
        }
    }

    /// Long/tidy-format table → `GridData` for `HeatmapChart` / `ContourChart`.
    ///
    /// Each row is one cell: integer row/col indices and a numeric value.
    /// Missing cells are filled with `fill_value` (use `0.0` or `f64::NAN`).
    ///
    /// ```text
    /// row | col | value
    /// ----+-----+------
    ///   0 |   0 |   0.8
    ///   0 |   1 |   0.3
    ///   1 |   0 |   0.5
    ///   1 |   1 |   1.0
    /// ```
    pub fn to_grid_long(
        &self,
        row_col: &str,
        col_col: &str,
        value_col: &str,
        fill_value: f64,
    ) -> GridData {
        let mut n_rows = 0usize;
        let mut n_cols = 0usize;
        let mut cells: Vec<(usize, usize, f64)> = Vec::new();

        for row in &self.rows {
            let Some(r) = row.get(row_col).and_then(FieldValue::as_f64) else {
                continue;
            };
            let Some(c) = row.get(col_col).and_then(FieldValue::as_f64) else {
                continue;
            };
            let Some(v) = row.get(value_col).and_then(FieldValue::as_f64) else {
                continue;
            };
            let (ri, ci) = (r as usize, c as usize);
            n_rows = n_rows.max(ri + 1);
            n_cols = n_cols.max(ci + 1);
            cells.push((ri, ci, v));
        }

        let mut values = vec![vec![fill_value; n_cols]; n_rows];
        for (ri, ci, v) in cells {
            values[ri][ci] = v;
        }

        GridData {
            values,
            row_labels: None,
            col_labels: None,
        }
    }

    // --- Internal helper ---

    fn extract_xy(&self, x_col: &str, y_col: &str) -> Vec<DataPoint> {
        self.rows
            .iter()
            .filter_map(|row| {
                let x = row.get(x_col)?.as_f64()?;
                let y = row.get(y_col)?.as_f64()?;
                Some(DataPoint::new(x, y))
            })
            .collect()
    }
}

// --- Convenience macros / helpers for building DataRow ---

/// Build a `DataRow` from key-value pairs.
///
/// ```rust,ignore
/// let row = data_row! { "date" => 1_000_000.0, "value" => 42.5, "label" => "A" };
/// ```
#[macro_export]
macro_rules! data_row {
    ($($key:expr => $val:expr),* $(,)?) => {{
        let mut row = $crate::core::field_value::DataRow::new();
        $(
            row.insert($key.to_owned(), $crate::core::field_value::FieldValue::from($val));
        )*
        row
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::encoding::{Encoding, Field};

    fn make_table() -> DataTable {
        let mut t = DataTable::default();
        for (x, y, cat) in [
            (1.0_f64, 10.0_f64, "A"),
            (2.0, 20.0, "B"),
            (3.0, 30.0, "A"),
            (4.0, 40.0, "B"),
        ] {
            let mut row = DataRow::new();
            row.insert("x".into(), FieldValue::Numeric(x));
            row.insert("y".into(), FieldValue::Numeric(y));
            row.insert("cat".into(), FieldValue::Text(cat.into()));
            t.push(row);
        }
        t
    }

    #[test]
    fn test_field_value_as_f64() {
        assert_eq!(FieldValue::Numeric(42.5).as_f64(), Some(42.5));
        assert_eq!(FieldValue::Timestamp(1000.0).as_f64(), Some(1000.0));
        assert_eq!(FieldValue::Bool(true).as_f64(), Some(1.0));
        assert_eq!(FieldValue::Bool(false).as_f64(), Some(0.0));
        assert_eq!(FieldValue::Text("hi".into()).as_f64(), None);
        assert_eq!(FieldValue::Null.as_f64(), None);
    }

    #[test]
    fn test_field_value_as_str() {
        assert_eq!(FieldValue::Text("hello".into()).as_str(), Some("hello"));
        assert_eq!(FieldValue::Numeric(1.0).as_str(), None);
    }

    #[test]
    fn test_extract_numeric() {
        let table = make_table();
        let xs = table.extract_numeric("x");
        assert_eq!(xs, vec![1.0, 2.0, 3.0, 4.0]);
    }

    #[test]
    fn test_extract_text() {
        let table = make_table();
        let cats = table.extract_text("cat");
        assert_eq!(cats, vec!["A", "B", "A", "B"]);
    }

    #[test]
    fn test_group_by() {
        let table = make_table();
        let groups = table.group_by("cat");
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].0, "A");
        assert_eq!(groups[0].1.len(), 2);
        assert_eq!(groups[1].0, "B");
        assert_eq!(groups[1].1.len(), 2);
    }

    #[test]
    fn test_to_dataset_no_color() {
        let table = make_table();
        let enc = Encoding::new(Field::quantitative("x"), Field::quantitative("y"));
        let dataset = table.to_dataset(&enc);
        assert_eq!(dataset.series.len(), 1);
        assert_eq!(dataset.series[0].data.len(), 4);
        assert_eq!(dataset.series[0].data[0], DataPoint::new(1.0, 10.0));
    }

    #[test]
    fn test_to_dataset_with_color() {
        let table = make_table();
        let enc = Encoding::new(Field::quantitative("x"), Field::quantitative("y"))
            .with_color(Field::nominal("cat"));
        let dataset = table.to_dataset(&enc);
        assert_eq!(dataset.series.len(), 2);
        // First group "A" has rows at x=1 and x=3
        assert_eq!(dataset.series[0].name, "A");
        assert_eq!(dataset.series[0].data.len(), 2);
        assert_eq!(dataset.series[1].name, "B");
        assert_eq!(dataset.series[1].data.len(), 2);
    }

    #[test]
    fn test_to_bar_dataset() {
        let mut t = DataTable::default();
        for (cat, val) in [("Q1", 100.0_f64), ("Q2", 150.0), ("Q3", 120.0)] {
            let mut row = DataRow::new();
            row.insert("period".into(), FieldValue::Text(cat.into()));
            row.insert("revenue".into(), FieldValue::Numeric(val));
            t.push(row);
        }
        let enc = Encoding::new(Field::nominal("period"), Field::quantitative("revenue"));
        let bd = t.to_bar_dataset(&enc);
        assert_eq!(bd.categories, vec!["Q1", "Q2", "Q3"]);
        assert_eq!(bd.series.len(), 1);
        assert_eq!(bd.series[0].values, vec![100.0, 150.0, 120.0]);
    }

    #[test]
    fn test_to_bar_dataset_multi_series() {
        let mut t = DataTable::default();
        for (cat, val, product) in [
            ("Q1", 100.0_f64, "A"),
            ("Q1", 80.0, "B"),
            ("Q2", 120.0, "A"),
            ("Q2", 90.0, "B"),
        ] {
            let mut row = DataRow::new();
            row.insert("period".into(), FieldValue::Text(cat.into()));
            row.insert("revenue".into(), FieldValue::Numeric(val));
            row.insert("product".into(), FieldValue::Text(product.into()));
            t.push(row);
        }
        let enc = Encoding::new(Field::nominal("period"), Field::quantitative("revenue"))
            .with_color(Field::nominal("product"));
        let bd = t.to_bar_dataset(&enc);
        assert_eq!(bd.categories, vec!["Q1", "Q2"]);
        assert_eq!(bd.series.len(), 2);
        let series_a = bd.series.iter().find(|s| s.name == "A").expect("series A");
        assert_eq!(series_a.values, vec![100.0, 120.0]);
    }

    #[test]
    fn test_from_rows_macro() {
        let row = crate::data_row! {
            "x" => 1.0_f64,
            "label" => "hello",
            "flag" => true,
        };
        assert_eq!(row.get("x"), Some(&FieldValue::Numeric(1.0)));
        assert_eq!(row.get("label"), Some(&FieldValue::Text("hello".into())));
        assert_eq!(row.get("flag"), Some(&FieldValue::Bool(true)));
    }

    #[test]
    fn test_empty_table() {
        let t = DataTable::default();
        assert!(t.is_empty());
        assert_eq!(t.len(), 0);
        let enc = Encoding::new(Field::quantitative("x"), Field::quantitative("y"));
        let ds = t.to_dataset(&enc);
        assert!(ds.series.is_empty() || ds.series[0].data.is_empty());
    }

    // --- Tests for new chart type conversions ---

    fn make_strip_table() -> DataTable {
        let mut t = DataTable::default();
        for (group, value) in [("A", 1.0), ("A", 2.0), ("B", 3.0), ("B", 4.0), ("B", 5.0)] {
            let mut row = DataRow::new();
            row.insert("group".into(), FieldValue::Text(group.into()));
            row.insert("value".into(), FieldValue::Numeric(value));
            t.push(row);
        }
        t
    }

    #[test]
    fn test_to_strip_groups_count() {
        let t = make_strip_table();
        let groups = t.to_strip_groups("group", "value");
        assert_eq!(groups.len(), 2);
    }

    #[test]
    fn test_to_strip_groups_values() {
        let t = make_strip_table();
        let groups = t.to_strip_groups("group", "value");
        let a = groups.iter().find(|g| g.name == "A").unwrap();
        assert_eq!(a.values, vec![1.0, 2.0]);
        let b = groups.iter().find(|g| g.name == "B").unwrap();
        assert_eq!(b.values, vec![3.0, 4.0, 5.0]);
    }

    #[test]
    fn test_to_sankey_infers_nodes() {
        let mut t = DataTable::default();
        for (src, dst, val) in [("Coal", "Electricity", 40.0), ("Gas", "Electricity", 30.0)] {
            let mut row = DataRow::new();
            row.insert("src".into(), FieldValue::Text(src.into()));
            row.insert("dst".into(), FieldValue::Text(dst.into()));
            row.insert("val".into(), FieldValue::Numeric(val));
            t.push(row);
        }
        let sankey = t.to_sankey("src", "dst", "val", None);
        // 3 unique labels: Coal, Gas, Electricity
        assert_eq!(sankey.nodes.len(), 3);
        assert_eq!(sankey.links.len(), 2);
        assert_eq!(sankey.links[0].value, 40.0);
    }

    #[test]
    fn test_to_sankey_link_indices() {
        let mut t = DataTable::default();
        let mut row = DataRow::new();
        row.insert("src".into(), FieldValue::Text("A".into()));
        row.insert("dst".into(), FieldValue::Text("B".into()));
        row.insert("val".into(), FieldValue::Numeric(10.0));
        t.push(row);
        let sankey = t.to_sankey("src", "dst", "val", None);
        // A=0, B=1
        assert_eq!(sankey.links[0].source, 0);
        assert_eq!(sankey.links[0].target, 1);
    }

    #[test]
    fn test_to_chord_matrix_shape() {
        let mut t = DataTable::default();
        for (label, v0, v1) in [("X", 0.0, 5.0), ("Y", 3.0, 0.0)] {
            let mut row = DataRow::new();
            row.insert("label".into(), FieldValue::Text(label.into()));
            row.insert("X".into(), FieldValue::Numeric(v0));
            row.insert("Y".into(), FieldValue::Numeric(v1));
            t.push(row);
        }
        let chord = t.to_chord_matrix("label", &["X", "Y"]);
        assert_eq!(chord.labels, vec!["X", "Y"]);
        assert_eq!(chord.matrix.len(), 2);
        assert_eq!(chord.matrix[0], vec![0.0, 5.0]);
        assert_eq!(chord.matrix[1], vec![3.0, 0.0]);
    }

    #[test]
    fn test_to_grid_wide_shape() {
        let mut t = DataTable::default();
        for (name, a, b) in [("R1", 1.0, 2.0), ("R2", 3.0, 4.0)] {
            let mut row = DataRow::new();
            row.insert("name".into(), FieldValue::Text(name.into()));
            row.insert("A".into(), FieldValue::Numeric(a));
            row.insert("B".into(), FieldValue::Numeric(b));
            t.push(row);
        }
        let grid = t.to_grid_wide(Some("name"), &["A", "B"]);
        assert_eq!(grid.values.len(), 2);
        assert_eq!(grid.values[0], vec![1.0, 2.0]);
        assert_eq!(
            grid.row_labels,
            Some(vec!["R1".to_owned(), "R2".to_owned()])
        );
        assert_eq!(grid.col_labels, Some(vec!["A".to_owned(), "B".to_owned()]));
    }

    #[test]
    fn test_to_grid_long_fills() {
        let mut t = DataTable::default();
        for (r, c, v) in [(0.0, 0.0, 1.0), (1.0, 1.0, 2.0)] {
            let mut row = DataRow::new();
            row.insert("row".into(), FieldValue::Numeric(r));
            row.insert("col".into(), FieldValue::Numeric(c));
            row.insert("val".into(), FieldValue::Numeric(v));
            t.push(row);
        }
        let grid = t.to_grid_long("row", "col", "val", 0.0);
        assert_eq!(grid.values.len(), 2);
        assert_eq!(grid.values[0], vec![1.0, 0.0]); // (0,0)=1, (0,1)=fill
        assert_eq!(grid.values[1], vec![0.0, 2.0]); // (1,0)=fill, (1,1)=2
    }
}

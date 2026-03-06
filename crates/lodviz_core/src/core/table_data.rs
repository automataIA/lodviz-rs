/// Data model and pure logic for the visual `DataTable` component.
///
/// This is distinct from [`crate::core::field_value::DataTable`], which is used
/// for converting tidy/long-format data into chart datasets. This module provides
/// a column-definition-aware model for displaying tabular data with sorting,
/// filtering, and conditional formatting.
use std::cmp::Ordering;

use crate::core::field_value::FieldValue;

// --- TableData ---

/// A fully-defined table: column schema + row data.
///
/// Each row is a `Vec<FieldValue>` indexed by column position,
/// matching the `columns` field in first-occurrence order.
#[derive(Debug, Clone, Default)]
pub struct TableData {
    /// Column definitions (schema).
    pub columns: Vec<ColumnDef>,
    /// Row data; `rows[i][j]` is the value at column `j` of row `i`.
    pub rows: Vec<Vec<FieldValue>>,
}

impl TableData {
    /// Create a new table from column definitions and pre-built rows.
    pub fn new(columns: Vec<ColumnDef>, rows: Vec<Vec<FieldValue>>) -> Self {
        Self { columns, rows }
    }

    /// Number of data rows.
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// Number of columns.
    pub fn col_count(&self) -> usize {
        self.columns.len()
    }

    /// Append a row.
    pub fn push_row(&mut self, row: Vec<FieldValue>) {
        self.rows.push(row);
    }

    /// Collect distinct text values from a column (for Category filter multiselect).
    pub fn distinct_values(&self, col_idx: usize) -> Vec<String> {
        let mut seen = std::collections::HashSet::new();
        let mut ordered = Vec::new();
        for row in &self.rows {
            if let Some(FieldValue::Text(s)) = row.get(col_idx) {
                if seen.insert(s.clone()) {
                    ordered.push(s.clone());
                }
            }
        }
        ordered.sort();
        ordered
    }
}

// --- ColumnDef ---

/// Definition for a single column in a `TableData`.
#[derive(Debug, Clone)]
pub struct ColumnDef {
    /// Machine-readable key (not displayed).
    pub key: String,
    /// Human-readable header label.
    pub label: String,
    /// Semantic data type (drives filter UI and sort comparator).
    pub col_type: ColumnType,
    /// Whether the column can be sorted by clicking the header.
    pub sortable: bool,
    /// Whether a filter control appears for this column.
    pub filterable: bool,
    /// Optional fixed pixel width (otherwise auto-sized).
    pub width: Option<u32>,
    /// Text alignment inside cells.
    pub alignment: Alignment,
    /// Optional conditional formatting rule.
    pub conditional: Option<ConditionalRule>,
}

impl ColumnDef {
    /// Convenience constructor with sensible defaults (sortable, filterable, left-aligned).
    pub fn new(key: impl Into<String>, label: impl Into<String>, col_type: ColumnType) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            col_type,
            sortable: true,
            filterable: true,
            width: None,
            alignment: Alignment::Left,
            conditional: None,
        }
    }

    /// Set text alignment (builder pattern).
    #[must_use]
    pub fn align(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Attach a conditional formatting rule (builder pattern).
    #[must_use]
    pub fn conditional(mut self, rule: ConditionalRule) -> Self {
        self.conditional = Some(rule);
        self
    }

    /// Disable sorting for this column (builder pattern).
    #[must_use]
    pub fn no_sort(mut self) -> Self {
        self.sortable = false;
        self
    }

    /// Disable the filter control for this column (builder pattern).
    #[must_use]
    pub fn no_filter(mut self) -> Self {
        self.filterable = false;
        self
    }
}

// --- ColumnType ---

/// Semantic type of a column, used to choose the appropriate filter UI.
#[derive(Debug, Clone, PartialEq)]
pub enum ColumnType {
    /// Free-form text (shows a text search input).
    Text,
    /// Numeric value (shows a min/max range input).
    Number,
    /// Boolean flag (shows a yes/no toggle).
    Boolean,
    /// Categorical value with a known set of options (shows a multiselect).
    Category(Vec<String>),
}

// --- Alignment ---

/// Horizontal text alignment for a column.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Alignment {
    #[default]
    Left,
    Center,
    Right,
}

impl Alignment {
    /// Returns the CSS `text-align` value.
    pub fn as_css(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Center => "center",
            Self::Right => "right",
        }
    }
}

// --- FilterOp ---

/// Comparison operator for numeric filters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareOp {
    /// Equal (=)
    Equal,
    /// Not equal (!=)
    NotEqual,
    /// Greater than (>)
    Greater,
    /// Greater than or equal (>=)
    GreaterEq,
    /// Less than (<)
    Less,
    /// Less than or equal (<=)
    LessEq,
}

impl CompareOp {
    /// Returns the symbol representation of the operator.
    pub fn symbol(self) -> &'static str {
        match self {
            Self::Equal => "=",
            Self::NotEqual => "≠",
            Self::Greater => ">",
            Self::GreaterEq => "≥",
            Self::Less => "<",
            Self::LessEq => "≤",
        }
    }

    /// Returns the label for the operator.
    pub fn label(self) -> &'static str {
        match self {
            Self::Equal => "Equal to",
            Self::NotEqual => "Not equal to",
            Self::Greater => "Greater than",
            Self::GreaterEq => "Greater than or equal to",
            Self::Less => "Less than",
            Self::LessEq => "Less than or equal to",
        }
    }

    /// Apply the comparison operator to two values.
    pub fn compare(self, a: f64, b: f64) -> bool {
        match self {
            Self::Equal => (a - b).abs() < f64::EPSILON,
            Self::NotEqual => (a - b).abs() >= f64::EPSILON,
            Self::Greater => a > b,
            Self::GreaterEq => a >= b,
            Self::Less => a < b,
            Self::LessEq => a <= b,
        }
    }
}

/// A filter predicate that can be applied to a single cell value.
#[derive(Debug, Clone)]
pub enum FilterOp {
    /// Cell text contains the given substring (case-insensitive).
    TextContains(String),
    /// Numeric value comparison with operator.
    NumberCompare { operator: CompareOp, value: f64 },
    /// Categorical value is in the selected set (empty = show all).
    CategoryIn(Vec<String>),
    /// Cell is null/empty.
    IsEmpty,
}

impl FilterOp {
    /// Returns `true` if `val` passes this filter.
    pub fn matches(&self, val: &FieldValue) -> bool {
        match self {
            Self::IsEmpty => val.is_null(),
            Self::TextContains(search) => {
                let lower = search.to_lowercase();
                match val {
                    FieldValue::Text(s) => s.to_lowercase().contains(&lower),
                    FieldValue::Numeric(n) => format!("{n}").contains(&lower),
                    FieldValue::Bool(b) => b.to_string().contains(&lower),
                    FieldValue::Timestamp(t) => format!("{t}").contains(&lower),
                    FieldValue::Null => false,
                }
            }
            Self::NumberCompare { operator, value } => match val.as_f64() {
                Some(n) => operator.compare(n, *value),
                None => false,
            },
            Self::CategoryIn(selected) => {
                if selected.is_empty() {
                    return true; // no items checked = show all
                }
                matches!(val, FieldValue::Text(s) if selected.contains(s))
            }
        }
    }

    /// Returns `true` when this filter would pass every possible value
    /// (i.e. it is effectively disabled and can be dropped).
    pub fn is_trivial(&self) -> bool {
        match self {
            Self::TextContains(s) => s.is_empty(),
            Self::NumberCompare { .. } => false, // all compare ops are meaningful
            Self::CategoryIn(items) => items.is_empty(),
            Self::IsEmpty => false,
        }
    }
}

// --- SortKey / SortDir ---

/// A single sort criterion: column index and direction.
#[derive(Debug, Clone)]
pub struct SortKey {
    /// Index into `TableData::columns`.
    pub col_index: usize,
    /// Ascending or descending.
    pub direction: SortDir,
}

/// Sort direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDir {
    Asc,
    Desc,
}

impl SortDir {
    /// Flip the direction.
    pub fn toggle(self) -> Self {
        match self {
            Self::Asc => Self::Desc,
            Self::Desc => Self::Asc,
        }
    }
}

// --- ConditionalRule ---

/// Visual encoding rule applied to cells in a column based on their numeric value.
#[derive(Debug, Clone)]
pub enum ConditionalRule {
    /// Background colour interpolated between `low` → optional `mid` → `high`.
    /// Colours are `#rrggbb` hex strings.
    ColorScale {
        low: String,
        mid: Option<String>,
        high: String,
    },
    /// A proportional bar rendered inside the cell.
    DataBar { color: String },
}

// --- Pure helper functions ---

/// Compare two optional `FieldValue`s for sorting; `None`/`Null` sorts last.
pub fn compare_field_values(a: Option<&FieldValue>, b: Option<&FieldValue>) -> Ordering {
    match (a, b) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Greater,
        (Some(_), None) => Ordering::Less,
        (Some(va), Some(vb)) => compare_values(va, vb),
    }
}

fn compare_values(a: &FieldValue, b: &FieldValue) -> Ordering {
    match (a, b) {
        (FieldValue::Numeric(x), FieldValue::Numeric(y))
        | (FieldValue::Timestamp(x), FieldValue::Timestamp(y)) => {
            x.partial_cmp(y).unwrap_or(Ordering::Equal)
        }
        (FieldValue::Text(x), FieldValue::Text(y)) => x.cmp(y),
        (FieldValue::Bool(x), FieldValue::Bool(y)) => x.cmp(y),
        (FieldValue::Null, FieldValue::Null) => Ordering::Equal,
        (FieldValue::Null, _) => Ordering::Greater,
        (_, FieldValue::Null) => Ordering::Less,
        _ => {
            let sa = sortable_string(a);
            let sb = sortable_string(b);
            sa.cmp(&sb)
        }
    }
}

fn sortable_string(val: &FieldValue) -> String {
    match val {
        FieldValue::Text(s) => s.clone(),
        FieldValue::Numeric(n) => format!("{n:020.6}"),
        FieldValue::Timestamp(t) => format!("{t:020.0}"),
        FieldValue::Bool(b) => b.to_string(),
        FieldValue::Null => String::new(),
    }
}

/// Format a `FieldValue` for display in a table cell.
pub fn format_cell_value(val: &FieldValue) -> String {
    match val {
        FieldValue::Text(s) => s.clone(),
        FieldValue::Numeric(n) => {
            if n.fract() == 0.0 && n.abs() < 1e15 {
                format!("{:.0}", n)
            } else {
                // Up to 4 decimal places, trailing zeros stripped
                format!("{:.4}", n)
                    .trim_end_matches('0')
                    .trim_end_matches('.')
                    .to_owned()
            }
        }
        FieldValue::Timestamp(t) => format!("{t}"),
        FieldValue::Bool(b) => if *b { "Yes" } else { "No" }.to_owned(),
        FieldValue::Null => String::new(),
    }
}

/// Compute the 0–1 normalised position of `val` within the column's numeric range.
/// Returns `0.0` for non-numeric values or when all values are equal.
pub fn data_bar_pct(val: &FieldValue, rows: &[Vec<FieldValue>], col_idx: usize) -> f64 {
    let v = match val.as_f64() {
        Some(n) => n,
        None => return 0.0,
    };

    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;
    for row in rows {
        if let Some(n) = row.get(col_idx).and_then(FieldValue::as_f64) {
            if n < min {
                min = n;
            }
            if n > max {
                max = n;
            }
        }
    }

    if !min.is_finite() || !max.is_finite() || (max - min).abs() < f64::EPSILON {
        return 1.0;
    }

    ((v - min) / (max - min)).clamp(0.0, 1.0)
}

/// Interpolate a CSS `rgb(r,g,b)` background colour for a ColorScale rule.
/// Returns `None` when any hex colour is unparseable or the value is non-numeric.
pub fn color_scale_bg(
    val: &FieldValue,
    rows: &[Vec<FieldValue>],
    col_idx: usize,
    low: &str,
    mid: Option<&str>,
    high: &str,
) -> Option<String> {
    let pct = {
        let v = val.as_f64()?;
        let mut mn = f64::INFINITY;
        let mut mx = f64::NEG_INFINITY;
        for row in rows {
            if let Some(n) = row.get(col_idx).and_then(FieldValue::as_f64) {
                if n < mn {
                    mn = n;
                }
                if n > mx {
                    mx = n;
                }
            }
        }
        if !mn.is_finite() || !mx.is_finite() || (mx - mn).abs() < f64::EPSILON {
            0.5
        } else {
            ((v - mn) / (mx - mn)).clamp(0.0, 1.0)
        }
    };

    let lo = parse_hex(low)?;
    let hi = parse_hex(high)?;

    let (r, g, b) = if let Some(mid_hex) = mid {
        let me = parse_hex(mid_hex)?;
        if pct < 0.5 {
            lerp_rgb(lo, me, pct * 2.0)
        } else {
            lerp_rgb(me, hi, (pct - 0.5) * 2.0)
        }
    } else {
        lerp_rgb(lo, hi, pct)
    };

    Some(format!("rgb({r},{g},{b})"))
}

fn parse_hex(hex: &str) -> Option<(u8, u8, u8)> {
    let h = hex.trim_start_matches('#');
    match h.len() {
        6 => Some((
            u8::from_str_radix(&h[0..2], 16).ok()?,
            u8::from_str_radix(&h[2..4], 16).ok()?,
            u8::from_str_radix(&h[4..6], 16).ok()?,
        )),
        3 => Some((
            u8::from_str_radix(&h[0..1].repeat(2), 16).ok()?,
            u8::from_str_radix(&h[1..2].repeat(2), 16).ok()?,
            u8::from_str_radix(&h[2..3].repeat(2), 16).ok()?,
        )),
        _ => None,
    }
}

fn lerp_rgb(a: (u8, u8, u8), b: (u8, u8, u8), t: f64) -> (u8, u8, u8) {
    let lerp = |x: u8, y: u8| (f64::from(x) + (f64::from(y) - f64::from(x)) * t) as u8;
    (lerp(a.0, b.0), lerp(a.1, b.1), lerp(a.2, b.2))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_text_contains_case_insensitive() {
        let op = FilterOp::TextContains("hello".to_owned());
        assert!(op.matches(&FieldValue::Text("Hello World".to_owned())));
        assert!(!op.matches(&FieldValue::Text("World".to_owned())));
        assert!(!op.matches(&FieldValue::Null));
    }

    #[test]
    fn filter_number_compare_operators() {
        // Greater than
        let op_gt = FilterOp::NumberCompare {
            operator: CompareOp::Greater,
            value: 10.0,
        };
        assert!(op_gt.matches(&FieldValue::Numeric(15.0)));
        assert!(!op_gt.matches(&FieldValue::Numeric(10.0)));
        assert!(!op_gt.matches(&FieldValue::Numeric(5.0)));

        // Less than or equal
        let op_le = FilterOp::NumberCompare {
            operator: CompareOp::LessEq,
            value: 20.0,
        };
        assert!(op_le.matches(&FieldValue::Numeric(20.0)));
        assert!(op_le.matches(&FieldValue::Numeric(15.0)));
        assert!(!op_le.matches(&FieldValue::Numeric(25.0)));

        // Equal
        let op_eq = FilterOp::NumberCompare {
            operator: CompareOp::Equal,
            value: 10.0,
        };
        assert!(op_eq.matches(&FieldValue::Numeric(10.0)));
        assert!(!op_eq.matches(&FieldValue::Numeric(10.1)));
        assert!(!op_eq.matches(&FieldValue::Text("10".to_owned())));
    }

    #[test]
    fn filter_category_empty_shows_all() {
        let op = FilterOp::CategoryIn(vec![]);
        assert!(op.matches(&FieldValue::Text("anything".to_owned())));
    }

    #[test]
    fn filter_category_in() {
        let op = FilterOp::CategoryIn(vec!["A".to_owned(), "B".to_owned()]);
        assert!(op.matches(&FieldValue::Text("A".to_owned())));
        assert!(!op.matches(&FieldValue::Text("C".to_owned())));
    }

    #[test]
    fn compare_numbers() {
        assert_eq!(
            compare_field_values(
                Some(&FieldValue::Numeric(1.0)),
                Some(&FieldValue::Numeric(2.0))
            ),
            Ordering::Less
        );
    }

    #[test]
    fn null_sorts_last() {
        assert_eq!(
            compare_field_values(Some(&FieldValue::Null), Some(&FieldValue::Numeric(1.0))),
            Ordering::Greater
        );
        assert_eq!(
            compare_field_values(None, Some(&FieldValue::Numeric(1.0))),
            Ordering::Greater
        );
    }

    #[test]
    fn format_cell_strips_trailing_zeros() {
        assert_eq!(format_cell_value(&FieldValue::Numeric(3.5)), "3.5");
        assert_eq!(format_cell_value(&FieldValue::Numeric(3.0)), "3");
        assert_eq!(format_cell_value(&FieldValue::Numeric(3.1234)), "3.1234");
    }

    #[test]
    fn data_bar_pct_normalises() {
        let rows = vec![
            vec![FieldValue::Numeric(0.0)],
            vec![FieldValue::Numeric(50.0)],
            vec![FieldValue::Numeric(100.0)],
        ];
        assert!((data_bar_pct(&FieldValue::Numeric(50.0), &rows, 0) - 0.5).abs() < 1e-9);
        assert!((data_bar_pct(&FieldValue::Numeric(100.0), &rows, 0) - 1.0).abs() < 1e-9);
        assert!((data_bar_pct(&FieldValue::Numeric(0.0), &rows, 0)).abs() < 1e-9);
    }
}

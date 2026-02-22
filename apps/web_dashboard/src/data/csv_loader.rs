use gloo_net::http::Request;
/// HTTP fetch + CSV-to-chart-type conversions
///
/// Each conversion function takes a `DataTable` (from `parse_csv`) and
/// returns the chart-specific data structure needed by lodviz_components.
use lodviz_components::components::charts::box_plot::BoxGroup;
use lodviz_components::components::charts::pie_chart::PieEntry;
use lodviz_components::components::charts::radar::RadarSeries;
use lodviz_core::core::csv::parse_csv;
use lodviz_core::core::data::DataPoint;
use lodviz_core::core::data::WaterfallKind;
use lodviz_core::core::data::{BarDataset, Dataset, OhlcBar, Series, WaterfallBar};
use lodviz_core::core::field_value::{DataTable, FieldValue};

/// Fetch a CSV file from `url` and parse it into a [`DataTable`].
pub async fn fetch_csv(url: &str) -> Result<DataTable, String> {
    let response = Request::get(url)
        .send()
        .await
        .map_err(|e| format!("HTTP error fetching {url}: {e}"))?;

    if !response.ok() {
        return Err(format!("HTTP {} for {url}", response.status()));
    }

    let text = response
        .text()
        .await
        .map_err(|e| format!("Failed to read response body: {e}"))?;

    parse_csv(&text)
}

// --- Conversion helpers ---

/// `DataTable(x, y)` → single-series `Dataset`
pub fn to_xy_dataset(table: &DataTable, x_col: &str, y_col: &str, name: &str) -> Dataset {
    let points: Vec<DataPoint> = table
        .rows()
        .iter()
        .filter_map(|row| {
            let x = row.get(x_col)?.as_f64()?;
            let y = row.get(y_col)?.as_f64()?;
            Some(DataPoint::new(x, y))
        })
        .collect();
    Dataset::from_series(Series::new(name, points))
}

/// `DataTable(x, y1, y2, …)` → multi-series `Dataset`
///
/// `series_names` pairs each y column with its display name.
pub fn to_multi_dataset(table: &DataTable, x_col: &str, series_names: &[(&str, &str)]) -> Dataset {
    let mut dataset = Dataset::new();
    for (y_col, name) in series_names {
        let points: Vec<DataPoint> = table
            .rows()
            .iter()
            .filter_map(|row| {
                let x = row.get(x_col)?.as_f64()?;
                let y = row.get(*y_col)?.as_f64()?;
                Some(DataPoint::new(x, y))
            })
            .collect();
        dataset.add_series(Series::new(*name, points));
    }
    dataset
}

/// Wide-format table → `BarDataset`
///
/// `cat_col` is the category column; `series_cols` maps value columns
/// to their display names.
pub fn to_grouped_bar(
    table: &DataTable,
    cat_col: &str,
    series_cols: &[(&str, &str)],
) -> BarDataset {
    let categories = table.extract_text(cat_col);
    let mut dataset = BarDataset::new(categories);
    for (col, name) in series_cols {
        let values = table.extract_numeric(col);
        dataset.add_series(*name, values);
    }
    dataset
}

/// Table(`label`, `value`) → `Vec<PieEntry>`
pub fn to_pie_entries(table: &DataTable, label_col: &str, value_col: &str) -> Vec<PieEntry> {
    table
        .rows()
        .iter()
        .filter_map(|row| {
            let label = row.get(label_col)?.as_str()?.to_owned();
            let value = row.get(value_col)?.as_f64()?;
            Some(PieEntry { label, value })
        })
        .collect()
}

/// Table(`group`, `value`) → `Vec<BoxGroup>` (used by BoxPlot and ViolinChart)
pub fn to_box_groups(table: &DataTable, group_col: &str, value_col: &str) -> Vec<BoxGroup> {
    let groups = table.group_by(group_col);
    groups
        .into_iter()
        .map(|(label, sub)| BoxGroup {
            label,
            data: sub.extract_numeric(value_col),
        })
        .collect()
}

/// Table(`value`) → `Vec<f64>` (used by Histogram)
pub fn to_histogram_values(table: &DataTable, col: &str) -> Vec<f64> {
    table.extract_numeric(col)
}

/// Table(`bar`, `open`, `high`, `low`, `close`) → `Vec<OhlcBar>`
pub fn to_ohlc(
    table: &DataTable,
    idx_col: &str,
    open_col: &str,
    high_col: &str,
    low_col: &str,
    close_col: &str,
) -> Vec<OhlcBar> {
    table
        .rows()
        .iter()
        .filter_map(|row| {
            let idx = row.get(idx_col)?.as_f64()?;
            let open = row.get(open_col)?.as_f64()?;
            let high = row.get(high_col)?.as_f64()?;
            let low = row.get(low_col)?.as_f64()?;
            let close = row.get(close_col)?.as_f64()?;
            Some(OhlcBar::new(idx, open, high, low, close))
        })
        .collect()
}

/// Wide-format table → `Vec<RadarSeries>`
///
/// First column is the series name; remaining columns are axis values
/// in the same order as the `axes` vec passed to `RadarChart`.
pub fn to_radar_series(table: &DataTable, name_col: &str, axis_cols: &[&str]) -> Vec<RadarSeries> {
    table
        .rows()
        .iter()
        .filter_map(|row| {
            let name = row.get(name_col)?.as_str()?.to_owned();
            let values: Vec<f64> = axis_cols
                .iter()
                .map(|col| row.get(*col).and_then(FieldValue::as_f64).unwrap_or(0.0))
                .collect();
            if values.is_empty() {
                None
            } else {
                Some(RadarSeries { name, values })
            }
        })
        .collect()
}

/// Table(`label`, `value`, `kind`) → `Vec<WaterfallBar>`
///
/// `kind` column must contain one of: `"start"`, `"delta"`, `"total"`.
pub fn to_waterfall_bars(
    table: &DataTable,
    label_col: &str,
    value_col: &str,
    kind_col: &str,
) -> Vec<WaterfallBar> {
    table
        .rows()
        .iter()
        .filter_map(|row| {
            let label = row.get(label_col)?.as_str()?.to_owned();
            let value = row.get(value_col)?.as_f64()?;
            let kind = match row.get(kind_col)?.as_str()? {
                "start" => WaterfallKind::Start,
                "total" => WaterfallKind::Total,
                _ => WaterfallKind::Delta,
            };
            Some(WaterfallBar { label, value, kind })
        })
        .collect()
}

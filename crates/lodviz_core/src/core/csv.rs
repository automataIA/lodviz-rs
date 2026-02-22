/// Minimal CSV parser: text → DataTable
///
/// - First non-empty, non-comment line → column headers
/// - Subsequent lines → DataRow with type-inferred values (Numeric or Text)
/// - Lines starting with `#` and empty lines are skipped
use std::collections::HashMap;

use crate::core::field_value::{DataTable, FieldValue};

/// Parse a CSV string into a [`DataTable`].
///
/// Returns `Err` if the input has no header row, otherwise each data row
/// is mapped to a `DataRow`. Cells are parsed as `f64` first; on failure
/// they become `FieldValue::Text`. Missing trailing columns are stored as
/// `FieldValue::Null`.
pub fn parse_csv(text: &str) -> Result<DataTable, String> {
    let mut lines = text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'));

    // First line is the header
    let header_line = lines
        .next()
        .ok_or_else(|| "CSV has no header row".to_string())?;

    let headers: Vec<&str> = header_line.split(',').map(str::trim).collect();

    let mut table = DataTable::default();

    for line in lines {
        let cells: Vec<&str> = line.split(',').collect();
        let mut row: HashMap<String, FieldValue> = HashMap::new();

        for (i, &col) in headers.iter().enumerate() {
            let value = match cells.get(i) {
                Some(cell) => {
                    let cell = cell.trim();
                    if cell.is_empty() {
                        FieldValue::Null
                    } else if let Ok(n) = cell.parse::<f64>() {
                        FieldValue::Numeric(n)
                    } else {
                        FieldValue::Text(cell.to_owned())
                    }
                }
                None => FieldValue::Null,
            };
            row.insert(col.to_owned(), value);
        }

        table.push(row);
    }

    Ok(table)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_csv_basic() {
        let csv = "x,y\n1.0,2.0\n3.0,4.0";
        let table = parse_csv(csv).unwrap();
        assert_eq!(table.len(), 2);
        assert_eq!(table.extract_numeric("x"), vec![1.0, 3.0]);
        assert_eq!(table.extract_numeric("y"), vec![2.0, 4.0]);
    }

    #[test]
    fn test_parse_csv_text_column() {
        let csv = "label,value\nApple,10.0\nBanana,20.0";
        let table = parse_csv(csv).unwrap();
        assert_eq!(table.extract_text("label"), vec!["Apple", "Banana"]);
        assert_eq!(table.extract_numeric("value"), vec![10.0, 20.0]);
    }

    #[test]
    fn test_parse_csv_skips_comments_and_empty() {
        let csv = "# This is a comment\nx,y\n\n1.0,2.0\n# another comment\n3.0,4.0\n";
        let table = parse_csv(csv).unwrap();
        assert_eq!(table.len(), 2);
    }

    #[test]
    fn test_parse_csv_missing_cells_become_null() {
        let csv = "a,b,c\n1.0,2.0";
        let table = parse_csv(csv).unwrap();
        let row = &table.rows()[0];
        assert_eq!(row.get("a"), Some(&FieldValue::Numeric(1.0)));
        assert_eq!(row.get("b"), Some(&FieldValue::Numeric(2.0)));
        assert_eq!(row.get("c"), Some(&FieldValue::Null));
    }

    #[test]
    fn test_parse_csv_empty_input_returns_err() {
        assert!(parse_csv("").is_err());
        assert!(parse_csv("# only comments\n").is_err());
    }
}

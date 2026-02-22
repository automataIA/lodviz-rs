/// Mark types for Grammar of Graphics
///
/// Marks define how data is visually represented (points, lines, areas, etc.)
/// Visual mark types for data representation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mark {
    /// Line connecting data points
    Line,
    /// Filled area under/between lines
    Area,
    /// Vertical or horizontal bars
    Bar,
    /// Individual points/circles
    Point,
    /// Circles with variable size
    Circle,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mark_equality() {
        assert_eq!(Mark::Line, Mark::Line);
        assert_ne!(Mark::Line, Mark::Bar);
    }

    #[test]
    fn test_mark_copy() {
        let mark1 = Mark::Point;
        let mark2 = mark1;
        assert_eq!(mark1, mark2);
    }

    #[test]
    fn test_mark_debug() {
        let mark = Mark::Area;
        let debug_str = format!("{:?}", mark);
        assert!(debug_str.contains("Area"));
    }
}

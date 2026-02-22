/// Encoding channels for Grammar of Graphics
///
/// Encodings map data fields to visual channels (x, y, color, size, etc.)
use super::data::DataType;

/// A field with its data type for encoding
#[derive(Debug, Clone)]
pub struct Field {
    /// Name of the field in the data
    pub name: String,
    /// Type of data (quantitative, temporal, nominal, ordinal)
    pub data_type: DataType,
}

impl Field {
    /// Create a new field
    pub fn new(name: impl Into<String>, data_type: DataType) -> Self {
        Self {
            name: name.into(),
            data_type,
        }
    }

    /// Create a quantitative field (continuous numerical)
    pub fn quantitative(name: impl Into<String>) -> Self {
        Self::new(name, DataType::Quantitative)
    }

    /// Create a temporal field (date/time)
    pub fn temporal(name: impl Into<String>) -> Self {
        Self::new(name, DataType::Temporal)
    }

    /// Create a nominal field (categorical, unordered)
    pub fn nominal(name: impl Into<String>) -> Self {
        Self::new(name, DataType::Nominal)
    }

    /// Create an ordinal field (categorical, ordered)
    pub fn ordinal(name: impl Into<String>) -> Self {
        Self::new(name, DataType::Ordinal)
    }
}

/// Encoding specification for mapping data to visual channels
#[derive(Debug, Clone)]
pub struct Encoding {
    /// X-axis encoding (required)
    pub x: Field,
    /// Y-axis encoding (required)
    pub y: Field,
    /// Color encoding (optional)
    pub color: Option<Field>,
    /// Size encoding (optional, for points/circles)
    pub size: Option<Field>,
}

impl Encoding {
    /// Create a new encoding with required x and y channels
    pub fn new(x: Field, y: Field) -> Self {
        Self {
            x,
            y,
            color: None,
            size: None,
        }
    }

    /// Set the color encoding channel
    pub fn with_color(mut self, color: Field) -> Self {
        self.color = Some(color);
        self
    }

    /// Set the size encoding channel
    pub fn with_size(mut self, size: Field) -> Self {
        self.size = Some(size);
        self
    }

    /// Set an optional color encoding channel (no-op when `None`)
    pub fn with_color_opt(self, color: Option<Field>) -> Self {
        match color {
            Some(f) => self.with_color(f),
            None => self,
        }
    }

    /// Set an optional size encoding channel (no-op when `None`)
    pub fn with_size_opt(self, size: Option<Field>) -> Self {
        match size {
            Some(f) => self.with_size(f),
            None => self,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_creation() {
        let field = Field::new("revenue", DataType::Quantitative);
        assert_eq!(field.name, "revenue");
        assert_eq!(field.data_type, DataType::Quantitative);
    }

    #[test]
    fn test_field_convenience_methods() {
        let quant = Field::quantitative("value");
        assert_eq!(quant.data_type, DataType::Quantitative);

        let temp = Field::temporal("date");
        assert_eq!(temp.data_type, DataType::Temporal);

        let nom = Field::nominal("category");
        assert_eq!(nom.data_type, DataType::Nominal);

        let ord = Field::ordinal("rank");
        assert_eq!(ord.data_type, DataType::Ordinal);
    }

    #[test]
    fn test_encoding_basic() {
        let encoding = Encoding::new(Field::quantitative("x"), Field::quantitative("y"));

        assert_eq!(encoding.x.name, "x");
        assert_eq!(encoding.y.name, "y");
        assert!(encoding.color.is_none());
        assert!(encoding.size.is_none());
    }

    #[test]
    fn test_encoding_with_color() {
        let encoding = Encoding::new(Field::quantitative("x"), Field::quantitative("y"))
            .with_color(Field::nominal("category"));

        assert!(encoding.color.is_some());
        assert_eq!(encoding.color.unwrap().name, "category");
    }

    #[test]
    fn test_encoding_with_size() {
        let encoding = Encoding::new(Field::quantitative("x"), Field::quantitative("y"))
            .with_size(Field::quantitative("magnitude"));

        assert!(encoding.size.is_some());
        assert_eq!(encoding.size.unwrap().name, "magnitude");
    }

    #[test]
    fn test_encoding_builder_pattern() {
        let encoding = Encoding::new(Field::quantitative("x"), Field::quantitative("y"))
            .with_color(Field::nominal("category"))
            .with_size(Field::quantitative("value"));

        assert!(encoding.color.is_some());
        assert!(encoding.size.is_some());
    }
}

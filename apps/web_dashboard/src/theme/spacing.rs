/// Spacing constants for consistent layout
///
/// Provides standard spacing values for margins, padding, and dimensions
///
/// Default chart dimensions
pub mod dimensions {
    /// Default chart width in pixels
    pub const CHART_WIDTH: u32 = 800;

    /// Default chart height in pixels
    pub const CHART_HEIGHT: u32 = 400;

    /// Small chart width
    pub const CHART_WIDTH_SM: u32 = 400;

    /// Small chart height
    pub const CHART_HEIGHT_SM: u32 = 200;

    /// Large chart width
    pub const CHART_WIDTH_LG: u32 = 1200;

    /// Large chart height
    pub const CHART_HEIGHT_LG: u32 = 600;
}

/// Standard margins for chart containers
pub mod margins {
    /// Minimal margin (for dense layouts)
    pub const MARGIN_MINIMAL: (f64, f64, f64, f64) = (10.0, 10.0, 20.0, 30.0); // top, right, bottom, left

    /// Standard margin (default)
    pub const MARGIN_STANDARD: (f64, f64, f64, f64) = (20.0, 20.0, 40.0, 50.0);

    /// Spacious margin (for presentations)
    pub const MARGIN_SPACIOUS: (f64, f64, f64, f64) = (30.0, 30.0, 60.0, 70.0);
}

/// Standard spacing values for UI elements
pub mod gap {
    /// Extra small spacing (2px)
    pub const XS: f64 = 2.0;

    /// Small spacing (4px)
    pub const SM: f64 = 4.0;

    /// Medium spacing (8px)
    pub const MD: f64 = 8.0;

    /// Large spacing (16px)
    pub const LG: f64 = 16.0;

    /// Extra large spacing (24px)
    pub const XL: f64 = 24.0;

    /// Extra extra large spacing (32px)
    pub const XXL: f64 = 32.0;
}

/// Typography sizes for chart labels
pub mod typography {
    /// Small text (10px)
    pub const TEXT_SM: f64 = 10.0;

    /// Normal text (12px)
    pub const TEXT_MD: f64 = 12.0;

    /// Large text (14px)
    pub const TEXT_LG: f64 = 14.0;

    /// Extra large text (16px)
    pub const TEXT_XL: f64 = 16.0;

    /// Title text (18px)
    pub const TEXT_TITLE: f64 = 18.0;
}

/// Stroke widths for lines and borders
pub mod stroke {
    /// Thin stroke (1px)
    pub const THIN: f64 = 1.0;

    /// Normal stroke (2px)
    pub const NORMAL: f64 = 2.0;

    /// Thick stroke (3px)
    pub const THICK: f64 = 3.0;

    /// Extra thick stroke (4px)
    pub const EXTRA_THICK: f64 = 4.0;
}

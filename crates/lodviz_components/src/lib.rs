//! # lodviz_components
//!
//! High-level components for data visualization, built on top of `lodviz_core`.
//!
//! This crate provides a suite of interactive, responsive chart components ready to be used
//! in Leptos applications. It includes support for light/dark theming, tooltips, and
//! composed dashboards.
//!
//! ## Modules
//!
//! - [`components::charts`]: Core chart components (Bar, Line, Scatter, Area, Box, etc.).
//! - [`components::interaction`]: Interactive elements like Zoom/Pan and Brushing.
//! - [`components::layout`]: Dashboard layout primitives strings.
//! - [`components::theme_provider`]: Global theme management.
//!
//! ## Example
//!
//! ```rust,ignore
//! use lodviz_components::components::charts::BarChart;
//! use lodviz_core::core::theme::ChartConfig;
//! use leptos::prelude::*;
//!
//! #[component]
//! fn MyChart() -> impl IntoView {
//!     view! {
//!         <BarChart
//!             data=Signal::derive(|| vec![/* ... */])
//!             config=Signal::derive(|| ChartConfig::default())
//!         />
//!     }
//! }
//! ```

/// Internal component subdivisions
pub mod components {
    /// Charting primitives and rendered SVG components
    pub mod charts;
    /// Interactive features like zoom, pan, and brushing
    pub mod interaction;
    /// Dashboard layout containers
    pub mod layout;
    /// Low-level SVG rendering utilities (axes, tooltips, legends)
    pub mod svg;
    /// Interactive BI-style data table
    pub mod table;
    /// Global theme management context
    pub mod theme_provider;
}

/// Leptos hooks for reactive utilities
pub mod hooks;

// Re-export specific components for easier access
pub use components::charts::area_chart::AreaChart;
pub use components::charts::bar_chart::BarChart;
pub use components::charts::box_plot::{BoxGroup, BoxPlot, ViolinChart};
pub use components::charts::candlestick::CandlestickChart;
pub use components::charts::histogram::Histogram;
pub use components::charts::line_chart::LineChart;
pub use components::charts::pie_chart::PieChart;
pub use components::charts::radar::{RadarChart, RadarSeries};
pub use components::charts::scatter_chart::ScatterChart;
pub use components::charts::smart_chart::SmartChart;
pub use components::charts::waterfall::WaterfallChart;
pub use components::interaction::brush::Brush;
pub use components::interaction::linked_context::{DashboardContext, LinkedDashboard};
pub use components::interaction::zoom_pan::{ZoomPan, ZoomTransform};
pub use components::layout::draggable_card::DraggableCard;
pub use components::svg::overlay::{SmaOverlay, TrendLine};
pub use components::theme_provider::ThemeProvider;
pub use lodviz_core::core::data::{BarDataset, BarSeries};

// New chart types
pub use components::charts::chord_chart::ChordChart;
pub use components::charts::contour_chart::ContourChart;
pub use components::charts::heatmap::HeatmapChart;
pub use components::charts::sankey_chart::SankeyChart;
pub use components::charts::strip_chart::{StripChart, StripLayout};

// New chart types (advanced)
pub use components::table::data_table::DataTable;

// New data types
pub use lodviz_core::core::color_map::{ColorMap, DivergingColorMap, SequentialColorMap};
pub use lodviz_core::core::data::{
    ChordData, GridData, SankeyData, SankeyLink, SankeyNode, StripGroup,
};
// Data table types
pub use lodviz_core::core::table_data::{
    Alignment, ColumnDef, ColumnType, ConditionalRule, FilterOp, SortDir, SortKey, TableData,
};

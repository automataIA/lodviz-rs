/// SVG rendering components for charts
/// Axis rendering primitives (ticks, labels, lines)
pub mod axis;
/// Tooltip specific for bar charts (categorical hit-testing)
pub mod bar_tooltip;
/// Tooltip specific for box plots and violin charts
pub mod box_violin_tooltip;
/// Tooltip specific for financial candlestick charts
pub mod candlestick_tooltip;
/// Vertical color bar (gradient + tick labels) for continuous color maps
pub mod colorbar;
/// Background grid lines rendering
pub mod grid;
/// Tooltip for heatmap cells (row/col/value)
pub mod heatmap_tooltip;
/// Chart legend rendering component
pub mod legend;
/// Selection overlays and crosshairs
pub mod overlay;
/// Tooltip specific for radar (spider) charts
pub mod radar_tooltip;
/// Tooltip for Sankey diagram nodes and links
pub mod sankey_tooltip;
/// General-purpose cartesian tooltip (line, scatter, area)
pub mod tooltip;
/// Tooltip specific for waterfall charts
pub mod waterfall_tooltip;

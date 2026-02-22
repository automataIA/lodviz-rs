/// SVG rendering components for charts
/// Axis rendering primitives (ticks, labels, lines)
pub mod axis;
/// Tooltip specific for bar charts (categorical hit-testing)
pub mod bar_tooltip;
/// Tooltip specific for box plots and violin charts
pub mod box_violin_tooltip;
/// Tooltip specific for financial candlestick charts
pub mod candlestick_tooltip;
/// Background grid lines rendering
pub mod grid;
/// Chart legend rendering component
pub mod legend;
/// Selection overlays and crosshairs
pub mod overlay;
/// Tooltip specific for radar (spider) charts
pub mod radar_tooltip;
/// General-purpose cartesian tooltip (line, scatter, area)
pub mod tooltip;
/// Tooltip specific for waterfall charts
pub mod waterfall_tooltip;

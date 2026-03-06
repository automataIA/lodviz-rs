/// Algorithms for data processing and optimization
pub mod arc;
/// Beeswarm / strip chart layout (jitter, beeswarm, center)
pub mod beeswarm;
/// Chord diagram layout (arc + ribbon paths)
pub mod chord_layout;
/// Marching squares contour extraction
pub mod contour;
/// Largest Triangle Three Buckets algorithm for downsampling
pub mod lttb;
/// Min-Max-Min-Max (M4) algorithm for extremely fast downsampling
pub mod m4;
/// Nearest neighbor search utilities
pub mod nearest;
/// Sankey flow diagram layout
pub mod sankey_layout;
/// Stacking algorithms for bar/area charts
pub mod stack;
/// Core statistical functions (mean, median, KDE, etc.)
pub mod statistics;

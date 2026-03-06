//! Interactive BI-style data table components.
//!
//! Only [`data_table::DataTable`] is public API — the sub-components are
//! implementation details used internally.

/// Filter popover (text / range / category)
pub(super) mod column_filter;
/// Column-specific filter popover (embedded in header)
pub(super) mod column_filter_popover;
/// Pagination footer
pub(super) mod table_footer;
/// Column header row with sort + filter controls
pub(super) mod table_header;
/// Theme support for dynamic styling
pub(in crate::components) mod table_theme;
/// Toolbar (global search + column visibility)
pub(super) mod table_toolbar;

/// Main data table component
pub mod data_table;

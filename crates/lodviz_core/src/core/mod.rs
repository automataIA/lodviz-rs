/// Core domain logic for data visualization
///
/// This module contains the fundamental data structures and traits
/// that form the foundation of the lodviz-rs library.
/// Accessibility primitives and structures
pub mod a11y;
/// CSV parsing utilities
pub mod csv;
/// Fundamental data abstractions
pub mod data;
/// Visual encoding specifications
pub mod encoding;
/// Typeless field value storage
pub mod field_value;
/// Rendering primitives representations
pub mod mark;
/// Data to screen mapping scales
pub mod scale;
/// Interactive selection definitions
pub mod selection;
/// Vega-lite inspired Chart specifications
pub mod spec;
/// Chart theming and styling configuration
pub mod theme;

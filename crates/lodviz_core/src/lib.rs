//! # lodviz_core
//!
//! Core visualization primitives and data structures for `lodviz-rs`.
//!
//! This crate provides the foundational building blocks for creating data visualizations,
//! including a tidy data model, grammar of graphics-inspired encodings, and statistical algorithms.
//!
//! ## Key Features
//!
//! - **Tidy Data Model**: Flexible `DataTable` and `DataRow` structures for heterogeneous data.
//! - **Grammar of Graphics**: `Encoding` and `Scale` types for mapping data to visual properties.
//! - **Statistical Algorithms**: KDE (Kernel Density Estimation), box plot statistics, and more.
//!
//! ## Example
//!
//! ```rust
//! use lodviz_core::core::field_value::{DataTable, DataRow, FieldValue};
//! use lodviz_core::core::encoding::{Encoding, Field};
//!
//! // Create a simple data table
//! let mut row: DataRow = DataRow::new();
//! row.insert("x".to_string(), FieldValue::Numeric(10.0));
//! row.insert("y".to_string(), FieldValue::Numeric(20.0));
//! let table = DataTable::from_rows(vec![row]);
//!
//! // Define an encoding
//! let enc = Encoding::new(Field::quantitative("x"), Field::quantitative("y"));
//! ```

/// Algorithms for data processing and downsampling
pub mod algorithms;
/// Core data structures, typestates, and theming
pub mod core;

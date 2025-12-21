//! XTV - X Tree Viewer
//!
//! A TUI application for viewing tree structures from serialized data files.

pub mod cli;
pub mod error;
pub mod parser;
pub mod tree;
pub mod ui;

pub use error::{Result, XtvError};

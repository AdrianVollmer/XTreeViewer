pub mod html;
pub mod json;
pub mod xml;

use crate::error::{Result, XtvError};
use crate::tree::Tree;
use std::path::Path;

/// Trait for parsing different file formats into a Tree
pub trait Parser {
    /// Parse content into a Tree structure
    fn parse(&self, content: &str) -> Result<Tree>;

    /// Check if this parser can handle the given file path
    fn can_parse(&self, file_path: &Path) -> bool;
}

/// Detect and return the appropriate parser for a file
pub fn detect_parser(file_path: &Path) -> Result<Box<dyn Parser>> {
    let extension = file_path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_lowercase());

    match extension.as_deref() {
        Some("json") => Ok(Box::new(json::JsonParser)),
        Some("xml") => Ok(Box::new(xml::XmlParser)),
        Some("html") | Some("htm") => Ok(Box::new(html::HtmlParser)),
        Some(ext) => Err(XtvError::UnsupportedFormat(format!(
            "File extension '.{}' is not supported",
            ext
        ))),
        None => Err(XtvError::UnsupportedFormat(
            "File has no extension".to_string(),
        )),
    }
}

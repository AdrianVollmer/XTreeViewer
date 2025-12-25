pub mod html;
pub mod json;
pub mod ldif;
pub mod xml;
pub mod yaml;

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
        Some("ldif") => Ok(Box::new(ldif::LdifParser)),
        Some("yaml") | Some("yml") => Ok(Box::new(yaml::YamlParser)),
        Some(ext) => Err(XtvError::UnsupportedFormat(format!(
            "File extension '.{}' is not supported",
            ext
        ))),
        None => Err(XtvError::UnsupportedFormat(
            "File has no extension".to_string(),
        )),
    }
}

/// Get parser from format string
pub fn get_parser_from_format(format: &str) -> Result<Box<dyn Parser>> {
    match format.to_lowercase().as_str() {
        "json" => Ok(Box::new(json::JsonParser)),
        "xml" => Ok(Box::new(xml::XmlParser)),
        "html" | "htm" => Ok(Box::new(html::HtmlParser)),
        "ldif" => Ok(Box::new(ldif::LdifParser)),
        "yaml" | "yml" => Ok(Box::new(yaml::YamlParser)),
        _ => Err(XtvError::UnsupportedFormat(format!(
            "Format '{}' is not supported",
            format
        ))),
    }
}

/// Detect parser from content by trying to identify the format
pub fn detect_parser_from_content(content: &str) -> Result<Box<dyn Parser>> {
    let trimmed = content.trim_start();

    // Try to detect format from content
    if trimmed.starts_with("<?xml") || trimmed.starts_with('<') {
        // Could be XML or HTML
        if trimmed.contains("<!DOCTYPE html") || trimmed.contains("<html") {
            Ok(Box::new(html::HtmlParser))
        } else {
            Ok(Box::new(xml::XmlParser))
        }
    } else if trimmed.starts_with('{') || trimmed.starts_with('[') {
        Ok(Box::new(json::JsonParser))
    } else if trimmed.starts_with("version:") || trimmed.starts_with("dn:") {
        Ok(Box::new(ldif::LdifParser))
    } else if trimmed.starts_with("---") || trimmed.starts_with("%YAML") {
        // YAML document separator or directive
        Ok(Box::new(yaml::YamlParser))
    } else if trimmed.contains(':') && !trimmed.contains("::") {
        // Generic YAML-like structure (key: value pairs)
        // Try YAML as it's more flexible than LDIF
        Ok(Box::new(yaml::YamlParser))
    } else {
        Err(XtvError::UnsupportedFormat(
            "Could not detect format from content. Use --format to specify the format.".to_string(),
        ))
    }
}

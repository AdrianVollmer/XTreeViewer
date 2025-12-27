pub mod html;
pub mod json;
pub mod jsonlines;
pub mod ldif;
pub mod toml;
pub mod xml;
pub mod yaml;

use crate::error::{Result, XtvError};
use crate::tree::Tree;
use std::path::Path;

/// Trait for parsing different file formats into a Tree.
///
/// Each file format (JSON, XML, YAML, LDIF, etc.) has its own parser implementation
/// that knows how to convert that format into XTV's tree structure.
///
/// # Implementing a Parser
///
/// To add support for a new format, implement this trait:
///
/// ```ignore
/// use xtv::parser::Parser;
/// use xtv::tree::Tree;
/// use xtv::error::Result;
/// use std::path::Path;
///
/// pub struct MyFormatParser;
///
/// impl Parser for MyFormatParser {
///     fn parse(&self, content: &str) -> Result<Tree> {
///         // Parse content and build tree
///         todo!()
///     }
///
///     fn can_parse(&self, file_path: &Path) -> bool {
///         file_path.extension()
///             .and_then(|ext| ext.to_str())
///             .map(|s| s.eq_ignore_ascii_case("myformat"))
///             .unwrap_or(false)
///     }
/// }
/// ```
pub trait Parser {
    /// Parses file content into a Tree structure.
    ///
    /// # Arguments
    ///
    /// * `content` - The complete file content as a string
    ///
    /// # Returns
    ///
    /// * `Ok(Tree)` - Successfully parsed tree
    /// * `Err(XtvError)` - Parse error with details
    ///
    /// # Errors
    ///
    /// Returns an error if the content is malformed or cannot be parsed.
    fn parse(&self, content: &str) -> Result<Tree>;

    /// Checks if this parser can handle the given file path.
    ///
    /// Typically checks the file extension to determine compatibility.
    ///
    /// # Arguments
    ///
    /// * `file_path` - Path to the file to check
    ///
    /// # Returns
    ///
    /// `true` if this parser can handle the file, `false` otherwise
    fn can_parse(&self, file_path: &Path) -> bool;
}

/// Detects and returns the appropriate parser for a file based on extension.
///
/// # Arguments
///
/// * `file_path` - Path to the file to detect parser for
///
/// # Returns
///
/// * `Ok(Box<dyn Parser>)` - Parser for the detected format
/// * `Err(XtvError::UnsupportedFormat)` - If the extension is not supported or missing
///
/// # Supported Extensions
///
/// - `.json` - JSON parser
/// - `.jsonl` - JSON Lines parser
/// - `.xml` - XML parser
/// - `.html`, `.htm` - HTML parser
/// - `.ldif` - LDIF parser
/// - `.toml` - TOML parser
/// - `.yaml`, `.yml` - YAML parser
///
/// # Examples
///
/// ```ignore
/// use xtv::parser::detect_parser;
/// use std::path::Path;
///
/// let parser = detect_parser(Path::new("data.json"))?;
/// let tree = parser.parse(content)?;
/// ```
pub fn detect_parser(file_path: &Path) -> Result<Box<dyn Parser>> {
    let extension = file_path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_lowercase());

    match extension.as_deref() {
        Some("json") => Ok(Box::new(json::JsonParser)),
        Some("jsonl") => Ok(Box::new(jsonlines::JsonLinesParser)),
        Some("xml") => Ok(Box::new(xml::XmlParser)),
        Some("html") | Some("htm") => Ok(Box::new(html::HtmlParser)),
        Some("ldif") => Ok(Box::new(ldif::LdifParser)),
        Some("toml") => Ok(Box::new(toml::TomlParser)),
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

/// Gets a parser from a format string.
///
/// This is useful when the user explicitly specifies the format via CLI flag
/// (e.g., `--format json`) instead of relying on file extension detection.
///
/// # Arguments
///
/// * `format` - Format name (case-insensitive)
///
/// # Returns
///
/// * `Ok(Box<dyn Parser>)` - Parser for the specified format
/// * `Err(XtvError::UnsupportedFormat)` - If the format is not recognized
///
/// # Supported Formats
///
/// - `"json"` - JSON parser
/// - `"jsonl"`, `"jsonlines"` - JSON Lines parser
/// - `"xml"` - XML parser
/// - `"html"`, `"htm"` - HTML parser
/// - `"ldif"` - LDIF parser
/// - `"toml"` - TOML parser
/// - `"yaml"`, `"yml"` - YAML parser
///
/// # Examples
///
/// ```ignore
/// use xtv::parser::get_parser_from_format;
///
/// let parser = get_parser_from_format("json")?;
/// let tree = parser.parse(content)?;
/// ```
pub fn get_parser_from_format(format: &str) -> Result<Box<dyn Parser>> {
    match format.to_lowercase().as_str() {
        "json" => Ok(Box::new(json::JsonParser)),
        "jsonl" | "jsonlines" => Ok(Box::new(jsonlines::JsonLinesParser)),
        "xml" => Ok(Box::new(xml::XmlParser)),
        "html" | "htm" => Ok(Box::new(html::HtmlParser)),
        "ldif" => Ok(Box::new(ldif::LdifParser)),
        "toml" => Ok(Box::new(toml::TomlParser)),
        "yaml" | "yml" => Ok(Box::new(yaml::YamlParser)),
        _ => Err(XtvError::UnsupportedFormat(format!(
            "Format '{}' is not supported",
            format
        ))),
    }
}

/// Detects parser from content by analyzing the file format.
///
/// This is used when reading from stdin or when the file has no extension.
/// The function examines the content's structure to guess the format.
///
/// # Arguments
///
/// * `content` - The file content to analyze
///
/// # Returns
///
/// * `Ok(Box<dyn Parser>)` - Parser for the detected format
/// * `Err(XtvError::UnsupportedFormat)` - If the format cannot be detected
///
/// # Detection Heuristics
///
/// - Starts with `<?xml` or `<` → XML or HTML (checks for DOCTYPE/html tag)
/// - Starts with `{` or `[` → JSON
/// - Starts with `version:` or `dn:` → LDIF
/// - Starts with `---` or `%YAML` → YAML
/// - Contains `:` but not `::` → YAML (fallback)
///
/// # Notes
///
/// Content detection is heuristic-based and may not be 100% accurate.
/// Use `--format` flag to explicitly specify the format when needed.
///
/// # Examples
///
/// ```ignore
/// use xtv::parser::detect_parser_from_content;
///
/// let content = "{\"key\": \"value\"}";
/// let parser = detect_parser_from_content(content)?;
/// let tree = parser.parse(content)?;
/// ```
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

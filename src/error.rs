use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum XtvError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("JSON parsing error: {0}")]
    JsonParse(#[from] serde_json::Error),

    #[error("YAML parsing error: {0}")]
    YamlParse(#[from] serde_yaml::Error),

    #[error("XML parsing error: {0}")]
    XmlParse(String),

    #[error("HTML parsing error: {0}")]
    HtmlParse(String),

    #[error("LDIF parsing error at line {line}: {message}")]
    LdifParse { line: usize, message: String },

    #[error("Unsupported file format: {0}")]
    UnsupportedFormat(String),

    #[error("Invalid node ID: {0}")]
    InvalidNodeId(usize),

    #[error("TUI error: {0}")]
    Tui(String),
}

pub type Result<T> = std::result::Result<T, XtvError>;

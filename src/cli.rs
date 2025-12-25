use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(name = "xtv")]
#[clap(version)]
#[clap(about = "X Tree Viewer - View tree structures from serialized data files", long_about = None)]
pub struct Cli {
    /// Path to the file to view (reads from stdin if not provided)
    #[clap(value_name = "FILE")]
    pub file: Option<PathBuf>,

    /// Format to use when reading from stdin (xml, json, html, ldif)
    #[clap(short, long, value_name = "FORMAT")]
    pub format: Option<String>,

    /// Threshold in bytes for switching to streaming mode (default: 100MB)
    #[clap(long, value_name = "BYTES", default_value = "104857600")]
    pub streaming_threshold: u64,

    /// Disable streaming mode (always load entire file into memory)
    #[clap(long)]
    pub no_streaming: bool,
}

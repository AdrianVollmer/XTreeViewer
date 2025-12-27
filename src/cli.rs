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

    /// Format to use when reading from stdin (xml, json, jsonl, html, ldif, toml, yaml)
    #[clap(short, long, value_name = "FORMAT")]
    pub format: Option<String>,

    /// Path to custom configuration file
    #[clap(short, long, value_name = "PATH")]
    pub config: Option<PathBuf>,

    /// Threshold in bytes for switching to streaming mode (overrides config)
    #[clap(long, value_name = "BYTES")]
    pub streaming_threshold: Option<u64>,

    /// Disable streaming mode (always load entire file into memory)
    #[clap(long)]
    pub no_streaming: bool,
}

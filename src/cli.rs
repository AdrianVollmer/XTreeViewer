use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(name = "xtv")]
#[clap(version)]
#[clap(about = "X Tree Viewer - View tree structures from serialized data files", long_about = None)]
pub struct Cli {
    /// Path to the file to view
    #[clap(value_name = "FILE")]
    pub file: PathBuf,
}

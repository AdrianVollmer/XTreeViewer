use clap::Parser;
use std::fs;
use xtv::{cli::Cli, parser, ui::App};

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> xtv::Result<()> {
    let cli = Cli::parse();

    // Read file content
    let content = fs::read_to_string(&cli.file)?;

    // Detect and create appropriate parser
    let parser = parser::detect_parser(&cli.file)?;

    // Parse into tree
    let tree = parser.parse(&content)?;

    // Run TUI
    let mut app = App::new(tree);
    app.run()?;

    Ok(())
}

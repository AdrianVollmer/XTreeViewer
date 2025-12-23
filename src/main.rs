use clap::Parser;
use std::fs;
use std::io::{self, Read};
use xtv::{cli::Cli, parser, ui::App};

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> xtv::Result<()> {
    let cli = Cli::parse();

    // Read content from file or stdin
    let content = if let Some(file_path) = &cli.file {
        fs::read_to_string(file_path)?
    } else {
        // Read from stdin
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer
    };

    // Detect and create appropriate parser
    let parser = if let Some(file_path) = &cli.file {
        // Use file extension
        parser::detect_parser(file_path)?
    } else if let Some(format) = &cli.format {
        // Use --format flag
        parser::get_parser_from_format(format)?
    } else {
        // Try to auto-detect from content
        parser::detect_parser_from_content(&content)?
    };

    // Parse into tree
    let tree = parser.parse(&content)?;

    // Run TUI
    let mut app = App::new(tree);
    app.run()?;

    Ok(())
}

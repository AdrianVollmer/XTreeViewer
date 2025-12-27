use clap::Parser;
use std::fs;
use std::io::{self, Read};
use xtv::{cli::Cli, config::Config, parser, tree::TreeVariant, ui::App};

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> xtv::Result<()> {
    let cli = Cli::parse();

    // Load configuration
    let config = Config::load_with_custom_path(cli.config.as_deref())?;

    // CLI flags override config values
    let streaming_threshold = cli
        .streaming_threshold
        .unwrap_or(config.streaming.threshold_bytes);
    let streaming_enabled = config.streaming.enabled && !cli.no_streaming;

    let tree_variant = if let Some(file_path) = &cli.file {
        // Check file size to determine if we should use streaming
        let metadata = fs::metadata(file_path)?;
        let file_size = metadata.len();

        let should_stream = streaming_enabled
            && file_size > streaming_threshold
            && file_path
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|s| s.eq_ignore_ascii_case("ldif"))
                .unwrap_or(false);

        if should_stream {
            // Use streaming mode for large LDIF files
            let streaming_tree = parser::ldif::build_ldif_index(file_path)?;
            TreeVariant::Streaming(streaming_tree)
        } else {
            // Use in-memory parsing
            let content = fs::read_to_string(file_path)?;
            let parser = parser::detect_parser(file_path)?;
            let tree = parser.parse(&content)?;
            TreeVariant::InMemory(tree)
        }
    } else {
        // Reading from stdin - always use in-memory mode
        let mut content = String::new();
        io::stdin().read_to_string(&mut content)?;

        let parser = if let Some(format) = &cli.format {
            parser::get_parser_from_format(format)?
        } else {
            parser::detect_parser_from_content(&content)?
        };

        let tree = parser.parse(&content)?;
        TreeVariant::InMemory(tree)
    };

    // Run TUI
    let mut app = App::new(tree_variant);
    app.run()?;

    Ok(())
}

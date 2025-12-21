# XTV - X Tree Viewer

A blazing fast TUI (Terminal User Interface) application for viewing tree structures from serialized data files.

## Features

- **Interactive TUI**: Navigate tree structures with keyboard controls
- **Multiple Format Support**: JSON, XML (with HTML and LDIF planned)
- **Read-Only Viewer**: Safe exploration of data files
- **Fast and Lightweight**: Written in Rust for performance

## Installation

```bash
cargo install --path .
```

## Usage

```bash
# View a JSON file
xtv examples/sample.json

# View an XML file
xtv examples/sample.xml
```

## Keyboard Controls

- **↑/↓**: Navigate up/down through nodes
- **Enter/→**: Expand selected node
- **←**: Collapse selected node
- **q**: Quit application

## Development

### Building from Source

```bash
cargo build --release
```

### Running Tests

```bash
cargo test
```

### Running the Binary

```bash
cargo run -- examples/sample.json
```

## Project Structure

- `src/tree/`: Abstract tree data structure
- `src/parser/`: Format-specific parsers (JSON, XML)
- `src/ui/`: TUI rendering and event handling
- `src/cli.rs`: Command-line interface
- `examples/`: Sample files for testing

## Roadmap

- [x] JSON parser
- [x] XML parser
- [ ] HTML parser
- [ ] LDIF parser
- [ ] Detail view for node attributes
- [ ] Search functionality
- [ ] Lazy loading for large datasets
- [ ] Streaming support for very large files

## License

MIT

## Contributing

See `CLAUDE.md` for development conventions and guidelines.

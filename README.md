# XTV - X Tree Viewer

A blazing fast TUI (Terminal User Interface) application for viewing
tree structures from serialized data files.

## Features

- **Interactive TUI**: Navigate tree structures with keyboard controls
- **Detail View**: View all attributes of selected nodes in a dedicated
  panel
- **Multiple Format Support**: JSON, JSON Lines, YAML, XML, HTML, LDIF
- **Read-Only Viewer**: Safe exploration of data files
- **Fast and Lightweight**: Written in Rust for performance

## Installation

``` bash
cargo install --path .
```

## Usage

``` bash
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

``` bash
cargo build --release
```

### Running Tests

``` bash
cargo test
```

### Running the Binary

``` bash
cargo run -- examples/sample.json
```

## Roadmap

- [x] JSON parser
- [x] XML parser
- [x] Detail view for node attributes
- [ ] HTML parser
- [ ] LDIF parser
- [ ] Search functionality
- [ ] Lazy loading for large datasets
- [ ] Streaming support for very large files

## License

MIT

## Contributing

See `CLAUDE.md` for development conventions and guidelines.

# XTV - X Tree Viewer

A blazing fast TUI (Terminal User Interface) application for viewing
tree structures from serialized data files.

## Features

- **Interactive TUI**: Navigate tree structures with keyboard controls
- **Multiple Format Support**: JSON, JSON Lines, YAML, TOML, XML, HTML,
  LDIF
- **Read-Only Viewer**: Safe exploration of data files
- **Fast and Lightweight**: Written in Rust for performance

**Disclaimer**: This was entirely vibe coded. I don't know any Rust. Do
with that information what you must.

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

## License

MIT

## Contributing

See `CONTRIBUTING.md` for development conventions and guidelines.

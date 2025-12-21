# X Tree Viewer

A TUI application to view tree structures from serialized data written
in Rust.

## Features

- Command name: `xtv`
- Display contents of the following file types:
  - JSON
  - XML
  - HTML
  - LDIF
- The TUI is interactive and resembles a classic pager.
- Blazing fast.
- Read-only.

## Implementation

- Written in Rust.
- For displaying files larger than some (configurable) limit of ~1G, use
  a streaming reader and/or lazy reading.
- When stream reading, it might be necessary to build an index first,
  especially for LDIF files, which can be up to ~20G.
- For each file format, build a reader that generates an abstract tree
  object, with optional streaming functionality.
- The tree structure knows nodes. Nodes can have attributes and a list
  of child nodes. In case of LDIF, attributes can be lists of values.
- By default, only the root node is expanded. When expanding any node,
  only load the first 100 child nodes and display a `[...]` if there are
  more nodes.
- Only a limited amount of attributes should be displayed in the main
  view. There should be a separate node view that displays a list of
  attributes.

## Conventions

- Code should be readable, maintainable, and testable.
- Try to adhere to the DRY principle.
- Don't overly abstract. Let's be pragmatic.
- Let's stick to best practices and idiomatic patterns.
- We prefer functions to be less than 50 lines and files less than 1000
  lines, but it's not a hard limit.
- Functions should not have more than five positional arguments, but
  it's not a hard limit.

## Development

- Issues will be in `issues/new` in markdown files.
- After solving an issue, move the file to `issues/closed`.
- After solving an issue, create a git commit. In the commit message,
  focus on the "why" instead of "how". The "how" can be deduced from the
  diff. However, a short summary of the "how" can't hurt to convey
  intent.
- Before commiting, run linters, formatters, and the test suite.
- When fixing bugs, add test cases.
- When adding features, update the docs and/or README.

## Agents

If you are an LLM:

- use
  `git -c user.name="Claude Code" -c user.email="noreply@anthropic.com"`
  when commiting.

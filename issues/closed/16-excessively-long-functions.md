# Excessively Long Functions

**Severity**: MEDIUM (Maintainability)
**Category**: Code Quality / Maintainability
**Locations**: Multiple

## Problem

Several functions are extremely long, making them difficult to understand, test, and maintain:

### 1. `build_ldif_index()` - 193 lines
**Location**: `src/parser/ldif.rs:355-546`

Mixes multiple responsibilities:
- I/O operations (file reading)
- Line folding handling
- DN parsing
- Attribute parsing
- Index structure building
- Progress indication

### 2. `handle_key()` - 222 lines
**Location**: `src/ui/app.rs:183-404`

Massive match statement with deeply nested logic for:
- Print popup handling
- Help screen handling
- Search mode handling
- Yank/copy commands (y prefix)
- Print commands (p prefix)
- Navigation keys
- All other key bindings

## Impact

- **Testing**: Hard to write unit tests for individual behaviors
- **Debugging**: Difficult to isolate issues
- **Maintenance**: Changes require understanding entire function
- **Code review**: Reviewers must hold too much context
- **Reusability**: Logic is trapped in monolithic function

## Recommendation

### For `build_ldif_index()`:

Extract into smaller functions:

```rust
fn build_ldif_index(file_path: &Path) -> Result<StreamingTree> {
    let (file, file_size) = open_ldif_file(file_path)?;
    let pb = create_progress_bar(file_size);

    let mut index = LdifIndex::new(0);
    let root_id = index.add_entry(IndexEntry::new(0, None, NodeType::Root));

    let reader = skip_version_line(file)?;
    let entries = parse_ldif_entries_for_index(reader, &pb)?;

    build_index_structure(&mut index, entries, root_id)?;

    pb.finish_with_message("Index complete");
    Ok(StreamingTree::new(file_path.to_path_buf(), index))
}

fn parse_ldif_entries_for_index(
    reader: Lines<BufReader<File>>,
    pb: &ProgressBar
) -> Result<Vec<IndexedEntry>> {
    // Extract entry parsing loop
}

fn build_index_structure(
    index: &mut LdifIndex,
    entries: Vec<IndexedEntry>,
    root_id: usize
) -> Result<()> {
    // Extract index building logic
}
```

### For `handle_key()`:

Extract command groups:

```rust
fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
    // Handle modal states first
    if self.print_content.is_some() {
        return self.handle_print_popup_key(key);
    }

    if self.show_help {
        return self.handle_help_key(key);
    }

    if self.search_mode {
        return self.handle_search_input_key(key);
    }

    // Handle prefix keys
    if self.last_key_was_y {
        return self.handle_yank_command(key);
    }

    if self.last_key_was_p {
        return self.handle_print_command(key);
    }

    // Handle normal navigation/command keys
    self.handle_normal_key(key)
}

fn handle_yank_command(&mut self, key: KeyEvent) -> Result<()> { /* ... */ }
fn handle_print_command(&mut self, key: KeyEvent) -> Result<()> { /* ... */ }
fn handle_navigation_key(&mut self, key: KeyCode) -> Result<()> { /* ... */ }
```

This makes the code much more modular and testable.

# Duplicate Page Navigation Code

**Severity**: LOW (DRY Violation)
**Category**: Code Quality / Maintainability
**Location**: `src/ui/app.rs:357-366` and `375-384`

## Problem

Identical code for page up/down navigation appears in two places:

### PageUp and '[' (lines 357-366)
```rust
KeyCode::PageUp => {
    for _ in 0..10 {
        self.tree_view.navigate_up();
    }
}
// ... other keys ...
KeyCode::Char('[') => {
    for _ in 0..10 {
        self.tree_view.navigate_up();
    }
}
```

### PageDown and ']' (lines 362-365, 380-383)
```rust
KeyCode::PageDown => {
    for _ in 0..10 {
        self.tree_view.navigate_down(&self.tree);
    }
}
// ... other keys ...
KeyCode::Char(']') => {
    for _ in 0..10 {
        self.tree_view.navigate_down(&self.tree);
    }
}
```

## Impact

- **Maintenance**: The magic number `10` appears 4 times and must be changed in all places
- **Consistency**: Risk of different behavior if one is updated but not others
- **Code bloat**: 8 lines that could be eliminated

## Recommendation

### Option 1: Combine cases

```rust
KeyCode::PageUp | KeyCode::Char('[') => {
    for _ in 0..PAGE_SCROLL_LINES {
        self.tree_view.navigate_up();
    }
}
KeyCode::PageDown | KeyCode::Char(']') => {
    for _ in 0..PAGE_SCROLL_LINES {
        self.tree_view.navigate_down(&self.tree);
    }
}
```

Where `PAGE_SCROLL_LINES` is a constant (see issue #17).

### Option 2: Helper method

```rust
fn page_up(&mut self) {
    for _ in 0..PAGE_SCROLL_LINES {
        self.tree_view.navigate_up();
    }
}

fn page_down(&mut self) {
    for _ in 0..PAGE_SCROLL_LINES {
        self.tree_view.navigate_down(&self.tree);
    }
}

// Then in handle_key:
KeyCode::PageUp | KeyCode::Char('[') => {
    self.page_up();
}
KeyCode::PageDown | KeyCode::Char(']') => {
    self.page_down();
}
```

### Option 3: Add method to TreeView

```rust
impl TreeView {
    pub fn page_up(&mut self, amount: usize) {
        for _ in 0..amount {
            self.navigate_up();
        }
    }

    pub fn page_down(&mut self, tree: &TreeVariant, amount: usize) {
        for _ in 0..amount {
            self.navigate_down(tree);
        }
    }
}

// Usage:
KeyCode::PageUp | KeyCode::Char('[') => {
    self.tree_view.page_up(PAGE_SCROLL_LINES);
}
KeyCode::PageDown | KeyCode::Char(']') => {
    self.tree_view.page_down(&self.tree, PAGE_SCROLL_LINES);
}
```

Option 1 is simplest and most idiomatic for Rust match statements.

# Path Recomputation on Every Frame

**Severity**: CRITICAL (Performance)
**Category**: Performance / Rendering
**Location**: `src/ui/app.rs:142-169`

## Problem

The `get_node_path()` method walks up the tree from the selected node to root on EVERY render:

```rust
fn get_node_path(&self) -> String {
    let selected_id = match self.tree_view.get_selected_node_id() {
        Some(id) => id,
        None => return String::new(),
    };

    // Build path from root to selected node
    let mut path_parts = Vec::new();
    let mut current_id = selected_id;

    // Walk up the tree to build the path
    loop {
        if let Some(node) = self.tree.get_node(current_id) {
            path_parts.push(node.label.clone());
        }

        // Find parent
        match self.tree.get_parent(current_id) {  // O(n) call in loop!
            Some(parent_id) => current_id = parent_id,
            None => break,
        }
    }
    // ...
}
```

This is called from `render()` which runs at up to 60fps.

## Impact

Combined with O(n) parent lookup (issue #12), this does O(n * depth) work on every single frame:

- Tree with 100k nodes, depth 10: ~1 million operations per frame
- At 60fps: 60 million operations per second
- Causes severe UI lag, high CPU usage, reduced battery life

## Recommendation

**Cache the path string** and invalidate only when selection changes:

```rust
pub struct App {
    // ... existing fields
    cached_path: String,
    last_selected_id: Option<usize>,
}

fn render(&mut self, frame: &mut ratatui::Frame) {
    // ... layout code

    // Update cache only if selection changed
    let current_id = self.tree_view.get_selected_node_id();
    if current_id != self.last_selected_id {
        self.cached_path = self.compute_node_path();
        self.last_selected_id = current_id;
    }

    let path_bar = Paragraph::new(&self.cached_path).style(...);
    // ...
}
```

This reduces path computation from 60fps to only when selection changes (typically 1-10 times per second).

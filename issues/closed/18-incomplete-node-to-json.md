# Incomplete Implementation of node_to_json

**Severity**: LOW (Functionality)
**Category**: Bug / Incomplete Feature
**Location**: `src/ui/app.rs:446-478`

## Problem

The `node_to_json()` function returns empty objects and arrays instead of building them from the tree:

```rust
fn node_to_json(&self, node: &crate::tree::TreeNode) -> Option<serde_json::Value> {
    use serde_json::{Map, Value};

    // ... handles attributes, text, comments correctly ...

    // For container nodes, build object or array
    if node.node_type == "object" {
        let map = Map::new();
        // Get children and build object
        // This is a simplified version - in reality we'd need to traverse children
        Some(Value::Object(map))  // Returns empty object!
    } else if node.node_type == "array" {
        Some(Value::Array(vec![]))  // Returns empty array!
    } else {
        Some(Value::String(node.label.clone()))
    }
}
```

## Impact

- **Broken feature**: The `yy` (copy value pretty) and `yv` (copy value compact) commands copy empty structures for objects and arrays
- **User confusion**: Users expect to copy the full JSON value but get `{}` or `[]`
- **Misleading**: Feature appears to work but produces incorrect output

## Usage

This function is called by:
- `get_node_value_pretty()` - used by `yy` copy command and `pp` print command
- `get_node_value_compact()` - used by `yv` copy command and `pv` print command

## Recommendation

Choose one of:

### Option 1: Implement fully

Recursively build JSON from tree structure:

```rust
fn node_to_json(&self, node: &crate::tree::TreeNode) -> Option<serde_json::Value> {
    use serde_json::{Map, Value};

    if node.is_attribute() {
        // ... existing code ...
    }

    if node.node_type == "text" || node.node_type == "comment" {
        // ... existing code ...
    }

    if node.node_type == "object" {
        let mut map = Map::new();
        let children = self.tree.get_children(node_id);
        for child_id in children {
            if let Some(child_node) = self.tree.get_node(child_id) {
                let value = self.node_to_json(&child_node)?;
                map.insert(child_node.label.clone(), value);
            }
        }
        return Some(Value::Object(map));
    }

    if node.node_type == "array" {
        let mut arr = Vec::new();
        let children = self.tree.get_children(node_id);
        for child_id in children {
            if let Some(child_node) = self.tree.get_node(child_id) {
                let value = self.node_to_json(&child_node)?;
                arr.push(value);
            }
        }
        return Some(Value::Array(arr));
    }

    Some(Value::String(node.label.clone()))
}
```

Note: This requires passing `node_id` in addition to `node` reference.

### Option 2: Remove the feature

If this isn't a core feature and is complex to implement correctly:
- Remove the `yy`, `yv`, `pp`, `pv` commands
- Keep only `ys`, `ps`, `yk`, `pk` which work correctly
- Document the limitation

### Option 3: Simplify to copy node label only

Document that copy commands only copy the label/key, not the full structure:
```rust
fn node_to_json(&self, node: &crate::tree::TreeNode) -> Option<serde_json::Value> {
    Some(Value::String(node.label.clone()))
}
```

The current state is misleading and should be fixed.

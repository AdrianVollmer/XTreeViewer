# O(n) Parent Lookup in In-Memory Tree

**Severity**: CRITICAL (Performance)
**Category**: Performance / Algorithm
**Location**: `src/tree/mod.rs:59-71`

## Problem

The `get_parent()` method scans through ALL nodes for every parent lookup:

```rust
pub fn get_parent(&self, child_id: usize) -> Option<usize> {
    if child_id == self.root_id {
        return None;
    }

    for (parent_id, node) in self.nodes.iter().enumerate() {  // O(n) scan!
        if node.children.contains(&child_id) {
            return Some(parent_id);
        }
    }

    None
}
```

## Impact

- **Performance**: Called frequently during:
  - Path building (every render!) - see `get_node_path()` in app.rs:159
  - Navigation operations
  - Search expand operations
- **Scaling**: For a tree with 100k nodes, this is catastrophic
- **UI responsiveness**: Causes visible lag and poor user experience

## Example

For a tree with 100,000 nodes and depth of 10:
- Path computation: O(n * depth) = 1,000,000 operations **per frame**
- At 60fps, this is 60 million operations per second just for displaying the path

## Recommendation

**Add parent pointers to TreeNode** (like StreamingTree already has):

1. Add `parent_id: Option<usize>` field to `TreeNode` struct
2. Set parent during tree construction when adding children
3. Make `get_parent()` a simple field lookup - O(1)

Example:
```rust
pub struct TreeNode {
    pub label: String,
    pub node_type: String,
    pub attributes: Vec<Attribute>,
    pub children: Vec<usize>,
    pub parent_id: Option<usize>,  // Add this
}

pub fn get_parent(&self, child_id: usize) -> Option<usize> {
    self.get_node(child_id).and_then(|node| node.parent_id)  // O(1)
}
```

This is the **single biggest performance improvement** available.

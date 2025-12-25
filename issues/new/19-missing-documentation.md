# Missing Documentation

**Severity**: LOW (Maintainability)
**Category**: Documentation
**Locations**: Throughout codebase

## Problem

Most public functions and types lack documentation comments (///) explaining:
- What they do
- What parameters mean
- What they return
- Error conditions
- Usage examples

## Examples of Undocumented Public APIs

### Tree module (src/tree/mod.rs)
```rust
pub struct Tree { ... }  // No doc comment

pub fn get_parent(&self, child_id: usize) -> Option<usize> {
    // What does this return? When is it None?
    // What's the performance characteristic?
}
```

### Streaming module (src/tree/streaming.rs)
```rust
pub struct StreamingTree { ... }  // No doc comment
pub struct LdifIndex { ... }  // No doc comment
pub struct IndexEntry { ... }  // No doc comment

pub fn load_node(&self, id: usize) -> Option<TreeNode> {
    // How does this work? What can fail?
    // Is it cached? What's the performance?
}
```

### Parser module (src/parser/ldif.rs)
```rust
pub fn build_ldif_index(file_path: &Path) -> Result<StreamingTree> {
    // This is a complex 193-line function with no documentation!
    // What index format? What are the performance characteristics?
    // When should this be used vs regular parsing?
}
```

### All Parser implementations
```rust
pub struct LdifParser;
pub struct XmlParser;
pub struct JsonParser;
// etc. - all lack documentation
```

## Impact

- **Onboarding**: New contributors must read implementation to understand API
- **Maintenance**: Easy to break contracts when they're not documented
- **API misuse**: Users might use APIs incorrectly
- **IDE experience**: No tooltips or inline help
- **`cargo doc`**: Generated documentation is sparse and unhelpful

## Recommendation

Add doc comments to all public items. Example:

```rust
/// Tree structure that stores nodes in a Vec for O(1) access by ID.
///
/// Nodes are stored in a flat vector and reference each other by index.
/// This provides efficient lookup but requires O(n) parent finding unless
/// parent pointers are added to nodes.
///
/// # Examples
///
/// ```
/// use xtv::tree::{Tree, TreeNode};
///
/// let root = TreeNode::new("root", "object");
/// let mut tree = Tree::new(root);
/// let child_id = tree.add_node(TreeNode::new("child", "string"));
/// ```
#[derive(Debug)]
pub struct Tree {
    nodes: Vec<TreeNode>,
    root_id: usize,
}

impl Tree {
    /// Find the parent of a given node by ID.
    ///
    /// This performs a linear scan through all nodes, checking if any
    /// node has `child_id` in its children list. This is O(n) complexity.
    ///
    /// # Arguments
    ///
    /// * `child_id` - The ID of the node whose parent to find
    ///
    /// # Returns
    ///
    /// * `Some(parent_id)` - The ID of the parent node
    /// * `None` - If the node is the root or parent is not found
    ///
    /// # Performance
    ///
    /// O(n) where n is the number of nodes in the tree.
    /// Consider adding parent pointers to TreeNode for O(1) lookup.
    pub fn get_parent(&self, child_id: usize) -> Option<usize> {
        // ...
    }
}
```

Priority areas:
1. Public structs and their purpose
2. Complex functions (>50 lines)
3. Functions with non-obvious behavior
4. Error conditions and edge cases
5. Performance characteristics

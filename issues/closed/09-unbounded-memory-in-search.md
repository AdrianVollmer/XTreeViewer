# Unbounded Memory Consumption in Search

**Severity**: CRITICAL (Security)
**Category**: Security / Performance
**Location**: `src/ui/app.rs:539-545`

## Problem

The `collect_all_nodes()` function recursively collects ALL nodes in the tree into memory:

```rust
fn collect_all_nodes(&self, node_id: usize) -> Vec<usize> {
    let mut nodes = vec![node_id];
    let children = self.tree.get_children(node_id);
    for child_id in children {
        nodes.extend(self.collect_all_nodes(child_id));  // Recursive, no limit
    }
    nodes
}
```

This is called during search operations (`perform_search()` on line 500).

## Impact

- **Security**: Denial of Service - For streaming trees designed to handle 20GB+ files, this defeats the entire streaming architecture and can cause out-of-memory crashes
- **Performance**: Application becomes unresponsive or crashes when searching large files
- **User Experience**: Search feature unusable on large files

## Recommendation

Implement one of the following solutions:

1. **Iterative search** that doesn't load all nodes at once:
   - Use depth-first traversal that yields matches lazily
   - Process nodes as you encounter them, don't collect them all first

2. **Add node count limits**:
   - Warn user if tree has > N nodes before searching
   - Only search first N nodes or visible nodes

3. **Streaming-aware search**:
   - For streaming trees, read and search the file directly
   - Don't expand the entire tree structure into memory

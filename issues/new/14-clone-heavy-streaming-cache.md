# Clone-Heavy Streaming Cache

**Severity**: MEDIUM (Performance)
**Category**: Performance / Memory
**Location**: `src/tree/streaming.rs:130, 138`

## Problem

Every cache access clones the entire TreeNode:

```rust
pub fn get_node(&self, id: usize) -> Option<TreeNode> {
    // Check cache first
    {
        let mut cache = self.cache.borrow_mut();
        if let Some(node) = cache.get(&id) {
            return Some(node.clone());  // Clones entire node
        }
    }

    // Load from disk
    if let Some(node) = self.load_node(id) {
        // Store in cache
        let mut cache = self.cache.borrow_mut();
        cache.put(id, node.clone());  // Another clone
        Some(node)
    } else {
        None
    }
}
```

## Impact

- **Performance**: TreeNode contains Strings and Vecs which are expensive to clone
- **Memory pressure**: Extra allocations and deallocations
- **CPU usage**: Unnecessary string copying on every cache hit
- **Cache efficiency**: The LRU cache design assumes O(1) access, but cloning adds overhead

## Example

A typical TreeNode might have:
- Label: ~50 bytes
- Node type: ~10 bytes
- Attributes: 1-10 attributes × ~100 bytes = 100-1000 bytes
- Children: ~10 IDs × 8 bytes = 80 bytes

Total: ~250-1150 bytes cloned on EVERY cache access.

With 1000-node cache, potential for 250KB-1MB of redundant cloning.

## Recommendation

**Use `Arc<TreeNode>` for reference-counted sharing**:

```rust
pub struct StreamingTree {
    file_path: PathBuf,
    index: LdifIndex,
    cache: std::cell::RefCell<LruCache<usize, Arc<TreeNode>>>,  // Arc instead of TreeNode
}

pub fn get_node(&self, id: usize) -> Option<Arc<TreeNode>> {
    {
        let mut cache = self.cache.borrow_mut();
        if let Some(node) = cache.get(&id) {
            return Some(Arc::clone(node));  // Just increments reference count
        }
    }

    if let Some(node) = self.load_node(id) {
        let node_arc = Arc::new(node);
        let mut cache = self.cache.borrow_mut();
        cache.put(id, Arc::clone(&node_arc));
        Some(node_arc)
    } else {
        None
    }
}
```

This changes expensive clones to cheap reference count increments.

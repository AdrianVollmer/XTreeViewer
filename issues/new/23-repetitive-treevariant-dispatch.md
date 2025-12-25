# Repetitive TreeVariant Dispatch Pattern

**Severity**: LOW (DRY Violation)
**Category**: Code Quality / Design Pattern
**Location**: `src/tree/mod.rs:82-121`

## Problem

Every method in `TreeVariant` follows the same repetitive pattern - just dispatching to the inner type:

```rust
impl TreeVariant {
    pub fn get_node(&self, id: usize) -> Option<TreeNode> {
        match self {
            TreeVariant::InMemory(tree) => tree.get_node(id).cloned(),
            TreeVariant::Streaming(tree) => tree.get_node(id),
        }
    }

    pub fn root_id(&self) -> usize {
        match self {
            TreeVariant::InMemory(tree) => tree.root_id(),
            TreeVariant::Streaming(tree) => tree.root_id(),
        }
    }

    pub fn get_children(&self, id: usize) -> Vec<usize> {
        match self {
            TreeVariant::InMemory(tree) => tree.get_children(id),
            TreeVariant::Streaming(tree) => tree.get_children(id),
        }
    }

    pub fn node_count(&self) -> usize {
        match self {
            TreeVariant::InMemory(tree) => tree.node_count(),
            TreeVariant::Streaming(tree) => tree.node_count(),
        }
    }

    pub fn get_parent(&self, child_id: usize) -> Option<usize> {
        match self {
            TreeVariant::InMemory(tree) => tree.get_parent(child_id),
            TreeVariant::Streaming(tree) => tree.get_parent(child_id),
        }
    }
}
```

## Impact

- **Boilerplate**: 5 methods × 5 lines = 25 lines of match statements
- **Maintenance**: Adding new methods requires boilerplate
- **Scalability**: Adding new tree types requires updating all methods
- **Verbosity**: Reduces code clarity

## Recommendation

### Option 1: Use trait object instead of enum

```rust
/// Trait for tree-like data structures
pub trait TreeLike: Debug {
    fn get_node(&self, id: usize) -> Option<TreeNode>;
    fn root_id(&self) -> usize;
    fn get_children(&self, id: usize) -> Vec<usize>;
    fn node_count(&self) -> usize;
    fn get_parent(&self, child_id: usize) -> Option<usize>;
}

impl TreeLike for Tree {
    fn get_node(&self, id: usize) -> Option<TreeNode> {
        self.get_node(id).cloned()
    }
    // ... implement others ...
}

impl TreeLike for StreamingTree {
    fn get_node(&self, id: usize) -> Option<TreeNode> {
        self.get_node(id)
    }
    // ... implement others ...
}

pub struct TreeVariant {
    inner: Box<dyn TreeLike>,
}

impl TreeVariant {
    // No more match statements - just delegate
    pub fn get_node(&self, id: usize) -> Option<TreeNode> {
        self.inner.get_node(id)
    }

    pub fn root_id(&self) -> usize {
        self.inner.root_id()
    }

    // ... etc.
}
```

### Option 2: Use macro to generate dispatch code

```rust
macro_rules! dispatch {
    ($self:expr, $method:ident $(, $arg:expr)*) => {
        match $self {
            TreeVariant::InMemory(tree) => tree.$method($($arg),*),
            TreeVariant::Streaming(tree) => tree.$method($($arg),*),
        }
    };
}

impl TreeVariant {
    pub fn root_id(&self) -> usize {
        dispatch!(self, root_id)
    }

    pub fn get_children(&self, id: usize) -> Vec<usize> {
        dispatch!(self, get_children, id)
    }

    pub fn get_parent(&self, child_id: usize) -> Option<usize> {
        dispatch!(self, get_parent, child_id)
    }

    // Special case for get_node due to clone
    pub fn get_node(&self, id: usize) -> Option<TreeNode> {
        match self {
            TreeVariant::InMemory(tree) => tree.get_node(id).cloned(),
            TreeVariant::Streaming(tree) => tree.get_node(id),
        }
    }
}
```

### Option 3: Accept the pattern as idiomatic

This pattern is actually common and idiomatic in Rust for sum types. If there are only 2 variants and a small number of methods, the explicit match might be clearest.

**Recommendation**: Option 1 (trait object) if extensibility is important, otherwise Option 3 (keep as-is) since it's clear and type-safe.

pub mod node;
pub mod streaming;

pub use node::{Attribute, TreeNode};
pub use streaming::{NodeType, StreamingTree};

/// Tree structure that stores nodes in a Vec for efficient O(1) access by ID.
///
/// Nodes are stored in a flat vector and reference each other by index (node ID).
/// This provides constant-time lookups but requires maintaining parent-child
/// relationships through ID references.
///
/// # Structure
///
/// - Nodes are stored in a `Vec<TreeNode>` with their position being their ID
/// - Each node contains a `children: Vec<usize>` list of child IDs
/// - Each node has a `parent_id: Option<usize>` for O(1) parent lookup
/// - The tree maintains a `root_id` pointing to the root node (typically 0)
///
/// # Examples
///
/// ```
/// use xtv::tree::{Tree, TreeNode};
///
/// let root = TreeNode::new("root", "object");
/// let mut tree = Tree::new(root);
///
/// // Add a child node
/// let child = TreeNode::new("child", "string");
/// let child_id = tree.add_child_node(0, child);
///
/// assert_eq!(tree.node_count(), 2);
/// ```
#[derive(Debug)]
pub struct Tree {
    nodes: Vec<TreeNode>,
    root_id: usize,
}

impl Tree {
    /// Creates a new tree with the given root node.
    ///
    /// The root node will be assigned ID 0.
    ///
    /// # Arguments
    ///
    /// * `root` - The root node for the tree
    ///
    /// # Examples
    ///
    /// ```
    /// use xtv::tree::{Tree, TreeNode};
    ///
    /// let root = TreeNode::new("root", "object");
    /// let tree = Tree::new(root);
    /// assert_eq!(tree.root_id(), 0);
    /// ```
    pub fn new(root: TreeNode) -> Self {
        Self {
            nodes: vec![root],
            root_id: 0,
        }
    }

    /// Adds a node to the tree and returns its ID.
    ///
    /// The node ID is simply the index in the internal vector, so this operation
    /// is O(1) amortized (accounting for vector growth).
    ///
    /// Note: This does not automatically establish parent-child relationships.
    /// Use `add_child_node()` for that purpose.
    ///
    /// # Arguments
    ///
    /// * `node` - The node to add
    ///
    /// # Returns
    ///
    /// The ID of the newly added node
    pub fn add_node(&mut self, node: TreeNode) -> usize {
        let id = self.nodes.len();
        self.nodes.push(node);
        id
    }

    /// Adds a node as a child of a parent node.
    ///
    /// This is the recommended way to add nodes as it automatically:
    /// - Sets the `parent_id` field on the child node
    /// - Adds the child ID to the parent's `children` list
    ///
    /// # Arguments
    ///
    /// * `parent_id` - The ID of the parent node
    /// * `node` - The child node to add
    ///
    /// # Returns
    ///
    /// The ID of the newly added child node
    ///
    /// # Panics
    ///
    /// Does not panic, but silently fails to update parent if `parent_id` is invalid.
    pub fn add_child_node(&mut self, parent_id: usize, mut node: TreeNode) -> usize {
        node.parent_id = Some(parent_id);
        let node_id = self.add_node(node);
        if let Some(parent) = self.get_node_mut(parent_id) {
            parent.children.push(node_id);
        }
        node_id
    }

    /// Gets a reference to a node by ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The node ID to look up
    ///
    /// # Returns
    ///
    /// * `Some(&TreeNode)` - If the node exists
    /// * `None` - If the ID is out of bounds
    ///
    /// # Performance
    ///
    /// O(1) - Direct vector indexing
    pub fn get_node(&self, id: usize) -> Option<&TreeNode> {
        self.nodes.get(id)
    }

    /// Gets a mutable reference to a node by ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The node ID to look up
    ///
    /// # Returns
    ///
    /// * `Some(&mut TreeNode)` - If the node exists
    /// * `None` - If the ID is out of bounds
    ///
    /// # Performance
    ///
    /// O(1) - Direct vector indexing
    pub fn get_node_mut(&mut self, id: usize) -> Option<&mut TreeNode> {
        self.nodes.get_mut(id)
    }

    /// Gets the root node ID.
    ///
    /// # Returns
    ///
    /// The ID of the root node (typically 0)
    pub fn root_id(&self) -> usize {
        self.root_id
    }

    /// Gets the children IDs of a node.
    ///
    /// # Arguments
    ///
    /// * `id` - The node ID whose children to retrieve
    ///
    /// # Returns
    ///
    /// A vector of child node IDs, or an empty vector if the node has no children
    /// or the node doesn't exist.
    ///
    /// # Performance
    ///
    /// O(k) where k is the number of children (due to cloning the children vector)
    pub fn get_children(&self, id: usize) -> Vec<usize> {
        self.get_node(id)
            .map(|node| node.children.clone())
            .unwrap_or_default()
    }

    /// Gets the total number of nodes in the tree.
    ///
    /// # Returns
    ///
    /// The count of all nodes including the root
    ///
    /// # Performance
    ///
    /// O(1) - Returns the length of the internal vector
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Finds the parent of a given node by ID.
    ///
    /// # Arguments
    ///
    /// * `child_id` - The ID of the node whose parent to find
    ///
    /// # Returns
    ///
    /// * `Some(parent_id)` - The ID of the parent node
    /// * `None` - If the node is the root, doesn't exist, or has no parent
    ///
    /// # Performance
    ///
    /// O(1) - Uses the parent_id field stored in each node
    pub fn get_parent(&self, child_id: usize) -> Option<usize> {
        self.get_node(child_id).and_then(|node| node.parent_id)
    }
}

/// Enum representing either an in-memory tree or a streaming tree.
///
/// XTV supports two modes of operation:
///
/// # InMemory Mode
///
/// All nodes are loaded into memory as a `Tree`. This provides the fastest
/// access but requires loading the entire file. Suitable for files up to
/// ~100MB or when working with smaller datasets.
///
/// # Streaming Mode
///
/// Nodes are loaded on-demand from disk using a `StreamingTree`. This enables
/// handling extremely large files (20GB+) by only keeping recently accessed
/// nodes in an LRU cache. Index building happens once at file load.
///
/// The mode is automatically selected based on file size, but can be controlled
/// via CLI flags: `--no-streaming` or `--streaming-threshold`.
///
/// Both variants implement the same interface, allowing code to work with either
/// mode transparently.
#[derive(Debug)]
pub enum TreeVariant {
    /// In-memory tree with all nodes loaded
    InMemory(Tree),
    /// Streaming tree with on-demand node loading
    Streaming(StreamingTree),
}

/// Macro to dispatch method calls to the appropriate tree variant.
///
/// This eliminates repetitive match boilerplate while maintaining type safety.
/// All methods that have identical signatures in both `Tree` and `StreamingTree`
/// can use this macro.
///
/// # Example
///
/// ```ignore
/// pub fn root_id(&self) -> usize {
///     dispatch!(self, root_id)
/// }
/// ```
///
/// Expands to:
///
/// ```ignore
/// match self {
///     TreeVariant::InMemory(tree) => tree.root_id(),
///     TreeVariant::Streaming(tree) => tree.root_id(),
/// }
/// ```
macro_rules! dispatch {
    ($self:expr, $method:ident $(, $arg:expr)*) => {
        match $self {
            TreeVariant::InMemory(tree) => tree.$method($($arg),*),
            TreeVariant::Streaming(tree) => tree.$method($($arg),*),
        }
    };
}

impl TreeVariant {
    /// Gets a node by ID, returning an owned copy.
    ///
    /// This method returns an owned `TreeNode` to provide a uniform interface
    /// across both in-memory and streaming variants.
    ///
    /// # Arguments
    ///
    /// * `id` - The node ID to look up
    ///
    /// # Returns
    ///
    /// * `Some(TreeNode)` - An owned copy of the node if it exists
    /// * `None` - If the node doesn't exist or failed to load
    ///
    /// # Performance
    ///
    /// - **InMemory**: O(1) lookup + clone of the node
    /// - **Streaming**: O(1) cache lookup or disk I/O + clone
    ///   - Cache hit: Fast Arc clone then node clone
    ///   - Cache miss: Blocking I/O to load from disk
    ///
    /// # Notes
    ///
    /// Both variants require cloning to return an owned TreeNode:
    /// - InMemory: Clones from `&TreeNode`
    /// - Streaming: Clones from `Arc<TreeNode>` (cache access via Arc::clone is cheap)
    pub fn get_node(&self, id: usize) -> Option<TreeNode> {
        match self {
            TreeVariant::InMemory(tree) => tree.get_node(id).cloned(),
            TreeVariant::Streaming(tree) => tree.get_node(id).map(|arc| (*arc).clone()),
        }
    }

    /// Gets the root node ID.
    ///
    /// # Returns
    ///
    /// The ID of the root node (typically 0)
    ///
    /// # Performance
    ///
    /// O(1) for both variants
    pub fn root_id(&self) -> usize {
        dispatch!(self, root_id)
    }

    /// Gets the children IDs of a node.
    ///
    /// # Arguments
    ///
    /// * `id` - The node ID whose children to retrieve
    ///
    /// # Returns
    ///
    /// A vector of child node IDs, or an empty vector if the node has no children
    /// or doesn't exist.
    ///
    /// # Performance
    ///
    /// O(k) where k is the number of children (due to cloning the children vector)
    pub fn get_children(&self, id: usize) -> Vec<usize> {
        dispatch!(self, get_children, id)
    }

    /// Gets the total number of nodes in the tree.
    ///
    /// # Returns
    ///
    /// The count of all nodes including the root
    ///
    /// # Performance
    ///
    /// O(1) for both variants
    pub fn node_count(&self) -> usize {
        dispatch!(self, node_count)
    }

    /// Finds the parent of a given node by ID.
    ///
    /// # Arguments
    ///
    /// * `child_id` - The ID of the node whose parent to find
    ///
    /// # Returns
    ///
    /// * `Some(parent_id)` - The ID of the parent node
    /// * `None` - If the node is the root, doesn't exist, or has no parent
    ///
    /// # Performance
    ///
    /// O(1) for both variants (uses parent_id field)
    pub fn get_parent(&self, child_id: usize) -> Option<usize> {
        dispatch!(self, get_parent, child_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_creation() {
        let root = TreeNode::new("root", "object");
        let tree = Tree::new(root);

        assert_eq!(tree.root_id(), 0);
        assert_eq!(tree.node_count(), 1);

        let root_node = tree.get_node(0).unwrap();
        assert_eq!(root_node.label, "root");
        assert_eq!(root_node.node_type, "object");
    }

    #[test]
    fn test_add_nodes() {
        let root = TreeNode::new("root", "object");
        let mut tree = Tree::new(root);

        let child1 = TreeNode::new("child1", "string");
        let child1_id = tree.add_node(child1);

        let child2 = TreeNode::new("child2", "number");
        let child2_id = tree.add_node(child2);

        // Add children to root
        tree.get_node_mut(0).unwrap().add_child(child1_id);
        tree.get_node_mut(0).unwrap().add_child(child2_id);

        assert_eq!(tree.node_count(), 3);

        let root_children = tree.get_children(0);
        assert_eq!(root_children.len(), 2);
        assert_eq!(root_children[0], child1_id);
        assert_eq!(root_children[1], child2_id);
    }
}

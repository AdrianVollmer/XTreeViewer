pub mod node;
pub mod streaming;

pub use node::{Attribute, TreeNode};
pub use streaming::{NodeType, StreamingTree};

/// Tree structure that stores nodes in a Vec for efficient access
#[derive(Debug)]
pub struct Tree {
    nodes: Vec<TreeNode>,
    root_id: usize,
}

impl Tree {
    /// Create a new tree with a root node
    pub fn new(root: TreeNode) -> Self {
        Self {
            nodes: vec![root],
            root_id: 0,
        }
    }

    /// Add a node to the tree and return its ID
    pub fn add_node(&mut self, node: TreeNode) -> usize {
        let id = self.nodes.len();
        self.nodes.push(node);
        id
    }

    /// Add a node as a child of a parent node
    /// This sets the parent_id on the child and adds it to the parent's children list
    pub fn add_child_node(&mut self, parent_id: usize, mut node: TreeNode) -> usize {
        node.parent_id = Some(parent_id);
        let node_id = self.add_node(node);
        if let Some(parent) = self.get_node_mut(parent_id) {
            parent.children.push(node_id);
        }
        node_id
    }

    /// Get a reference to a node by ID
    pub fn get_node(&self, id: usize) -> Option<&TreeNode> {
        self.nodes.get(id)
    }

    /// Get a mutable reference to a node by ID
    pub fn get_node_mut(&mut self, id: usize) -> Option<&mut TreeNode> {
        self.nodes.get_mut(id)
    }

    /// Get the root node ID
    pub fn root_id(&self) -> usize {
        self.root_id
    }

    /// Get the children IDs of a node
    pub fn get_children(&self, id: usize) -> Vec<usize> {
        self.get_node(id)
            .map(|node| node.children.clone())
            .unwrap_or_default()
    }

    /// Get the total number of nodes in the tree
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Find the parent of a given node by ID
    /// Returns None if the node is the root or parent is not found
    /// This is now O(1) using the parent_id field
    pub fn get_parent(&self, child_id: usize) -> Option<usize> {
        self.get_node(child_id).and_then(|node| node.parent_id)
    }
}

/// Enum representing either an in-memory tree or a streaming tree
#[derive(Debug)]
pub enum TreeVariant {
    InMemory(Tree),
    Streaming(StreamingTree),
}

impl TreeVariant {
    /// Get a reference to a node by ID
    pub fn get_node(&self, id: usize) -> Option<TreeNode> {
        match self {
            TreeVariant::InMemory(tree) => tree.get_node(id).cloned(),
            TreeVariant::Streaming(tree) => tree.get_node(id),
        }
    }

    /// Get the root node ID
    pub fn root_id(&self) -> usize {
        match self {
            TreeVariant::InMemory(tree) => tree.root_id(),
            TreeVariant::Streaming(tree) => tree.root_id(),
        }
    }

    /// Get the children IDs of a node
    pub fn get_children(&self, id: usize) -> Vec<usize> {
        match self {
            TreeVariant::InMemory(tree) => tree.get_children(id),
            TreeVariant::Streaming(tree) => tree.get_children(id),
        }
    }

    /// Get the total number of nodes in the tree
    pub fn node_count(&self) -> usize {
        match self {
            TreeVariant::InMemory(tree) => tree.node_count(),
            TreeVariant::Streaming(tree) => tree.node_count(),
        }
    }

    /// Find the parent of a given node by ID
    pub fn get_parent(&self, child_id: usize) -> Option<usize> {
        match self {
            TreeVariant::InMemory(tree) => tree.get_parent(child_id),
            TreeVariant::Streaming(tree) => tree.get_parent(child_id),
        }
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

/// Represents a single attribute of a tree node
#[derive(Debug, Clone)]
pub struct Attribute {
    pub key: String,
    pub value: String,
}

impl Attribute {
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }
}

/// Represents a node in the tree structure
#[derive(Debug, Clone)]
pub struct TreeNode {
    /// Display label for this node
    pub label: String,

    /// Node type (e.g., "object", "array", "element", "text")
    pub node_type: String,

    /// Attributes associated with this node
    pub attributes: Vec<Attribute>,

    /// Child node IDs (indices into the tree's node vector)
    pub children: Vec<usize>,
}

impl TreeNode {
    pub fn new(label: impl Into<String>, node_type: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            node_type: node_type.into(),
            attributes: Vec::new(),
            children: Vec::new(),
        }
    }

    pub fn with_attributes(mut self, attributes: Vec<Attribute>) -> Self {
        self.attributes = attributes;
        self
    }

    pub fn add_attribute(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.attributes.push(Attribute::new(key, value));
    }

    pub fn add_child(&mut self, child_id: usize) {
        self.children.push(child_id);
    }

    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }

    pub const VIRTUAL_ATTRIBUTES_TYPE: &'static str = "@attributes";
    pub const ATTRIBUTE_TYPE: &'static str = "attribute";

    pub fn is_virtual_attributes(&self) -> bool {
        self.node_type == Self::VIRTUAL_ATTRIBUTES_TYPE
    }

    pub fn is_attribute(&self) -> bool {
        self.node_type == Self::ATTRIBUTE_TYPE
    }
}

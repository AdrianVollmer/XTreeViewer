/// Represents a single attribute of a tree node.
///
/// Attributes are key-value pairs attached to nodes. They're used to store:
/// - XML/HTML element attributes
/// - LDIF entry attributes
/// - JSON object properties (when not displayed as child nodes)
/// - General metadata about nodes
///
/// # Examples
///
/// ```
/// use xtv::tree::node::Attribute;
///
/// let attr = Attribute::new("id", "123");
/// assert_eq!(attr.key, "id");
/// assert_eq!(attr.value, "123");
/// ```
#[derive(Debug, Clone)]
pub struct Attribute {
    /// The attribute key/name
    pub key: String,
    /// The attribute value
    pub value: String,
}

impl Attribute {
    /// Creates a new attribute.
    ///
    /// # Arguments
    ///
    /// * `key` - The attribute key/name (converted to String)
    /// * `value` - The attribute value (converted to String)
    ///
    /// # Examples
    ///
    /// ```
    /// use xtv::tree::node::Attribute;
    ///
    /// // From &str
    /// let attr1 = Attribute::new("name", "value");
    ///
    /// // From String
    /// let attr2 = Attribute::new("key".to_string(), "val".to_string());
    /// ```
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }
}

/// Represents a node in the tree structure.
///
/// TreeNode is the fundamental building block of the tree. Each node has:
/// - A label (display name)
/// - A type (e.g., "object", "array", "element", "text")
/// - Optional attributes (key-value pairs)
/// - References to child nodes (by ID)
/// - Reference to parent node (by ID)
///
/// # Node Types
///
/// Common node types include:
/// - `"object"` - JSON object, XML/HTML element
/// - `"array"` - JSON array
/// - `"element"` - XML/HTML element
/// - `"text"` - Text content node
/// - `"comment"` - Comment node
/// - `"entry"` - LDIF entry
/// - `"attribute"` - Individual attribute value
/// - `"@attributes"` - Virtual container for attributes (see [`VIRTUAL_ATTRIBUTES_TYPE`](TreeNode::VIRTUAL_ATTRIBUTES_TYPE))
///
/// # Examples
///
/// ```
/// use xtv::tree::TreeNode;
///
/// // Create a simple node
/// let mut node = TreeNode::new("myObject", "object");
///
/// // Add attributes
/// node.add_attribute("id", "123");
/// node.add_attribute("name", "example");
///
/// // Check type
/// assert!(!node.is_virtual_attributes());
/// assert!(!node.is_attribute());
/// ```
#[derive(Debug, Clone)]
pub struct TreeNode {
    /// Display label for this node
    pub label: String,

    /// Node type (e.g., "object", "array", "element", "text", "entry")
    pub node_type: String,

    /// Attributes associated with this node (key-value pairs)
    pub attributes: Vec<Attribute>,

    /// Child node IDs (indices into the tree's node vector)
    pub children: Vec<usize>,

    /// Parent node ID (None for root node)
    pub parent_id: Option<usize>,
}

impl TreeNode {
    /// Creates a new tree node.
    ///
    /// # Arguments
    ///
    /// * `label` - The display label for this node (converted to String)
    /// * `node_type` - The node type (converted to String)
    ///
    /// # Examples
    ///
    /// ```
    /// use xtv::tree::TreeNode;
    ///
    /// let node = TreeNode::new("root", "object");
    /// assert_eq!(node.label, "root");
    /// assert_eq!(node.node_type, "object");
    /// assert!(node.attributes.is_empty());
    /// assert!(node.children.is_empty());
    /// ```
    pub fn new(label: impl Into<String>, node_type: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            node_type: node_type.into(),
            attributes: Vec::new(),
            children: Vec::new(),
            parent_id: None,
        }
    }

    /// Creates a node with a specific set of attributes (builder pattern).
    ///
    /// # Arguments
    ///
    /// * `attributes` - Vector of attributes to set
    ///
    /// # Examples
    ///
    /// ```
    /// use xtv::tree::{TreeNode, Attribute};
    ///
    /// let attrs = vec![
    ///     Attribute::new("id", "1"),
    ///     Attribute::new("name", "test"),
    /// ];
    /// let node = TreeNode::new("item", "element").with_attributes(attrs);
    /// assert_eq!(node.attributes.len(), 2);
    /// ```
    pub fn with_attributes(mut self, attributes: Vec<Attribute>) -> Self {
        self.attributes = attributes;
        self
    }

    /// Adds an attribute to this node.
    ///
    /// # Arguments
    ///
    /// * `key` - The attribute key/name (converted to String)
    /// * `value` - The attribute value (converted to String)
    ///
    /// # Examples
    ///
    /// ```
    /// use xtv::tree::TreeNode;
    ///
    /// let mut node = TreeNode::new("item", "element");
    /// node.add_attribute("id", "123");
    /// node.add_attribute("enabled", "true");
    /// assert_eq!(node.attributes.len(), 2);
    /// ```
    pub fn add_attribute(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.attributes.push(Attribute::new(key, value));
    }

    /// Adds a child node ID to this node's children list.
    ///
    /// Note: This only updates the children list. It does not set the parent_id
    /// on the child node. Use [`Tree::add_child_node`](crate::tree::Tree::add_child_node)
    /// for proper parent-child relationship setup.
    ///
    /// # Arguments
    ///
    /// * `child_id` - The ID of the child node to add
    pub fn add_child(&mut self, child_id: usize) {
        self.children.push(child_id);
    }

    /// Checks if this node has any children.
    ///
    /// # Returns
    ///
    /// `true` if the node has at least one child, `false` otherwise
    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }

    /// The node type string for virtual attribute container nodes.
    ///
    /// Virtual attribute nodes are created to hold individual attributes as children,
    /// providing a consistent tree structure for displaying attributes in the UI.
    pub const VIRTUAL_ATTRIBUTES_TYPE: &'static str = "@attributes";

    /// The node type string for individual attribute nodes.
    ///
    /// Attribute nodes represent single key-value pairs and are typically children
    /// of a virtual attributes container node.
    pub const ATTRIBUTE_TYPE: &'static str = "attribute";

    /// Checks if this node is a virtual attributes container.
    ///
    /// # Returns
    ///
    /// `true` if node_type equals [`VIRTUAL_ATTRIBUTES_TYPE`](TreeNode::VIRTUAL_ATTRIBUTES_TYPE)
    pub fn is_virtual_attributes(&self) -> bool {
        self.node_type == Self::VIRTUAL_ATTRIBUTES_TYPE
    }

    /// Checks if this node is an individual attribute.
    ///
    /// # Returns
    ///
    /// `true` if node_type equals [`ATTRIBUTE_TYPE`](TreeNode::ATTRIBUTE_TYPE)
    pub fn is_attribute(&self) -> bool {
        self.node_type == Self::ATTRIBUTE_TYPE
    }
}

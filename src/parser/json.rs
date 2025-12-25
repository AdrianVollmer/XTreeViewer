use super::Parser;
use crate::error::Result;
use crate::tree::{Tree, TreeNode};
use serde_json::Value;
use std::path::Path;

pub struct JsonParser;

impl Parser for JsonParser {
    fn parse(&self, content: &str) -> Result<Tree> {
        let value: Value = serde_json::from_str(content)?;
        let mut tree = Tree::new(TreeNode::new("root", "root"));
        let root_id = tree.root_id();

        // Build tree from JSON value - handle top level specially
        match &value {
            Value::Object(map) => {
                // Add object fields directly to root
                for (key, child_value) in map {
                    convert_value(&mut tree, root_id, child_value, key);
                }
            }
            Value::Array(arr) => {
                // Add array items directly to root
                for (index, item) in arr.iter().enumerate() {
                    convert_value(&mut tree, root_id, item, &format!("[{}]", index));
                }
            }
            _ => {
                // For scalar values at top level, add them as a child
                convert_value(&mut tree, root_id, &value, "value");
            }
        }

        Ok(tree)
    }

    fn can_parse(&self, file_path: &Path) -> bool {
        file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|s| s.eq_ignore_ascii_case("json"))
            .unwrap_or(false)
    }
}

fn convert_value(tree: &mut Tree, parent_id: usize, value: &Value, key: &str) {
    match value {
        Value::Object(map) => {
            // Create a node for this object
            let mut node = TreeNode::new(key, "object");

            // Add attribute for object size
            node.add_attribute("size", format!("{} fields", map.len()));

            let node_id = tree.add_child_node(parent_id, node);

            // Recursively add children
            for (child_key, child_value) in map {
                convert_value(tree, node_id, child_value, child_key);
            }
        }
        Value::Array(arr) => {
            // Create a node for this array
            let mut node = TreeNode::new(key, "array");
            node.add_attribute("size", format!("{} items", arr.len()));

            let node_id = tree.add_child_node(parent_id, node);

            // Recursively add children with indices
            for (index, item) in arr.iter().enumerate() {
                convert_value(tree, node_id, item, &format!("[{}]", index));
            }
        }
        Value::String(s) => {
            let mut node = TreeNode::new(key, TreeNode::ATTRIBUTE_TYPE);
            node.add_attribute("value", s.clone());
            tree.add_child_node(parent_id, node);
        }
        Value::Number(n) => {
            let mut node = TreeNode::new(key, TreeNode::ATTRIBUTE_TYPE);
            node.add_attribute("value", n.to_string());
            tree.add_child_node(parent_id, node);
        }
        Value::Bool(b) => {
            let mut node = TreeNode::new(key, TreeNode::ATTRIBUTE_TYPE);
            node.add_attribute("value", b.to_string());
            tree.add_child_node(parent_id, node);
        }
        Value::Null => {
            let mut node = TreeNode::new(key, TreeNode::ATTRIBUTE_TYPE);
            node.add_attribute("value", "null");
            tree.add_child_node(parent_id, node);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_json() {
        let json = r#"{"name": "test", "count": 42}"#;
        let parser = JsonParser;
        let tree = parser.parse(json).unwrap();

        assert!(tree.node_count() > 0);
        let root = tree.get_node(tree.root_id()).unwrap();
        assert_eq!(root.label, "root");
    }

    #[test]
    fn test_parse_nested_json() {
        let json = r#"{"user": {"name": "Alice", "age": 30}}"#;
        let parser = JsonParser;
        let tree = parser.parse(json).unwrap();

        assert!(tree.node_count() > 3);
    }
}

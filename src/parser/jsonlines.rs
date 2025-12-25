use super::Parser;
use crate::error::Result;
use crate::tree::{Tree, TreeNode};
use serde_json::Value;
use std::path::Path;

pub struct JsonLinesParser;

impl Parser for JsonLinesParser {
    fn parse(&self, content: &str) -> Result<Tree> {
        let mut tree = Tree::new(TreeNode::new("root", "root"));
        let root_id = tree.root_id();

        // Parse each line as a separate JSON value
        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            // Skip empty lines
            if trimmed.is_empty() {
                continue;
            }

            // Parse the JSON value on this line
            let value: Value = serde_json::from_str(trimmed)?;

            // Create a node for this line, numbered starting from 1
            let label = format!("[{}]", line_num + 1);
            convert_value(&mut tree, root_id, &value, &label);
        }

        Ok(tree)
    }

    fn can_parse(&self, file_path: &Path) -> bool {
        file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|s| s.eq_ignore_ascii_case("jsonl"))
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
    fn test_parse_simple_jsonlines() {
        let jsonl = r#"{"name": "Alice", "age": 30}
{"name": "Bob", "age": 25}
{"name": "Charlie", "age": 35}"#;
        let parser = JsonLinesParser;
        let tree = parser.parse(jsonl).unwrap();

        assert!(tree.node_count() > 0);
        let root = tree.get_node(tree.root_id()).unwrap();
        assert_eq!(root.label, "root");
        // Should have 3 children (one for each line)
        assert_eq!(root.children.len(), 3);
    }

    #[test]
    fn test_parse_jsonlines_with_empty_lines() {
        let jsonl = r#"{"name": "Alice"}

{"name": "Bob"}
"#;
        let parser = JsonLinesParser;
        let tree = parser.parse(jsonl).unwrap();

        let root = tree.get_node(tree.root_id()).unwrap();
        // Should have 2 children (empty line is skipped)
        assert_eq!(root.children.len(), 2);
    }

    #[test]
    fn test_parse_jsonlines_different_types() {
        let jsonl = r#"{"type": "object"}
["array", "values"]
"just a string"
42
true
null"#;
        let parser = JsonLinesParser;
        let tree = parser.parse(jsonl).unwrap();

        let root = tree.get_node(tree.root_id()).unwrap();
        // Should have 6 children (one for each line)
        assert_eq!(root.children.len(), 6);
    }

    #[test]
    fn test_jsonlines_line_numbers() {
        let jsonl = r#"{"id": 1}
{"id": 2}
{"id": 3}"#;
        let parser = JsonLinesParser;
        let tree = parser.parse(jsonl).unwrap();

        let root = tree.get_node(tree.root_id()).unwrap();

        // First child should be labeled [1]
        let first_child = tree.get_node(root.children[0]).unwrap();
        assert_eq!(first_child.label, "[1]");

        // Second child should be labeled [2]
        let second_child = tree.get_node(root.children[1]).unwrap();
        assert_eq!(second_child.label, "[2]");

        // Third child should be labeled [3]
        let third_child = tree.get_node(root.children[2]).unwrap();
        assert_eq!(third_child.label, "[3]");
    }

    #[test]
    fn test_can_parse_jsonl_extension() {
        let parser = JsonLinesParser;
        assert!(parser.can_parse(Path::new("test.jsonl")));
        assert!(parser.can_parse(Path::new("test.JSONL")));
        assert!(!parser.can_parse(Path::new("test.json")));
        assert!(!parser.can_parse(Path::new("test.txt")));
    }

    #[test]
    fn test_parse_nested_jsonlines() {
        let jsonl = r#"{"user": {"name": "Alice", "age": 30}, "active": true}
{"user": {"name": "Bob", "age": 25}, "active": false}"#;
        let parser = JsonLinesParser;
        let tree = parser.parse(jsonl).unwrap();

        // Should create a complex tree structure
        assert!(tree.node_count() > 10);
    }
}

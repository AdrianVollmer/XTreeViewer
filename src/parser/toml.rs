use super::Parser;
use crate::error::Result;
use crate::tree::{Tree, TreeNode};
use std::path::Path;
use toml::Value;

pub struct TomlParser;

impl Parser for TomlParser {
    fn parse(&self, content: &str) -> Result<Tree> {
        let value: Value = toml::from_str(content)?;
        let mut tree = Tree::new(TreeNode::new("root", "root"));
        let root_id = tree.root_id();

        // TOML documents are always tables at the top level
        if let Value::Table(table) = &value {
            for (key, child_value) in table {
                convert_value(&mut tree, root_id, child_value, key);
            }
        }

        Ok(tree)
    }

    fn can_parse(&self, file_path: &Path) -> bool {
        file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|s| s.eq_ignore_ascii_case("toml"))
            .unwrap_or(false)
    }
}

fn convert_value(tree: &mut Tree, parent_id: usize, value: &Value, key: &str) {
    match value {
        Value::Table(table) => {
            // Create a node for this table
            let mut node = TreeNode::new(key, "table");
            node.add_attribute("size", format!("{} fields", table.len()));

            let node_id = tree.add_child_node(parent_id, node);

            // Recursively add children
            for (child_key, child_value) in table {
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
        Value::Integer(n) => {
            let mut node = TreeNode::new(key, TreeNode::ATTRIBUTE_TYPE);
            node.add_attribute("value", n.to_string());
            tree.add_child_node(parent_id, node);
        }
        Value::Float(f) => {
            let mut node = TreeNode::new(key, TreeNode::ATTRIBUTE_TYPE);
            node.add_attribute("value", f.to_string());
            tree.add_child_node(parent_id, node);
        }
        Value::Boolean(b) => {
            let mut node = TreeNode::new(key, TreeNode::ATTRIBUTE_TYPE);
            node.add_attribute("value", b.to_string());
            tree.add_child_node(parent_id, node);
        }
        Value::Datetime(dt) => {
            let mut node = TreeNode::new(key, TreeNode::ATTRIBUTE_TYPE);
            node.add_attribute("value", dt.to_string());
            tree.add_child_node(parent_id, node);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_toml() {
        let toml = r#"
name = "test"
count = 42
"#;
        let parser = TomlParser;
        let tree = parser.parse(toml).unwrap();

        assert!(tree.node_count() > 0);
        let root = tree.get_node(tree.root_id()).unwrap();
        assert_eq!(root.label, "root");
    }

    #[test]
    fn test_parse_nested_toml() {
        let toml = r#"
[user]
name = "Alice"
age = 30
"#;
        let parser = TomlParser;
        let tree = parser.parse(toml).unwrap();

        assert!(tree.node_count() > 3);
    }

    #[test]
    fn test_parse_toml_array() {
        let toml = r#"
numbers = [1, 2, 3, 4, 5]
"#;
        let parser = TomlParser;
        let tree = parser.parse(toml).unwrap();

        assert!(tree.node_count() > 5);
    }

    #[test]
    fn test_parse_toml_types() {
        let toml = r#"
string = "hello"
integer = 42
float = 3.14
boolean = true
datetime = 1979-05-27T07:32:00Z
"#;
        let parser = TomlParser;
        let tree = parser.parse(toml).unwrap();

        assert!(tree.node_count() > 5);
    }

    #[test]
    fn test_can_parse_toml_extension() {
        let parser = TomlParser;
        assert!(parser.can_parse(Path::new("config.toml")));
        assert!(parser.can_parse(Path::new("settings.TOML")));
        assert!(!parser.can_parse(Path::new("data.json")));
    }
}

use super::Parser;
use crate::error::Result;
use crate::tree::{Tree, TreeNode};
use serde_yaml::Value;
use std::path::Path;

pub struct YamlParser;

impl Parser for YamlParser {
    fn parse(&self, content: &str) -> Result<Tree> {
        let value: Value = serde_yaml::from_str(content)?;
        let mut tree = Tree::new(TreeNode::new("root", "root"));
        let root_id = tree.root_id();

        // Build tree from YAML value - handle top level specially
        match &value {
            Value::Mapping(map) => {
                // Add mapping fields directly to root
                for (key, child_value) in map {
                    let key_str = match key {
                        Value::String(s) => s.clone(),
                        Value::Number(n) => n.to_string(),
                        Value::Bool(b) => b.to_string(),
                        Value::Null => "null".to_string(),
                        _ => format!("{:?}", key),
                    };
                    convert_value(&mut tree, root_id, child_value, &key_str);
                }
            }
            Value::Sequence(arr) => {
                // Add sequence items directly to root
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
            .map(|s| s.eq_ignore_ascii_case("yaml") || s.eq_ignore_ascii_case("yml"))
            .unwrap_or(false)
    }
}

fn convert_value(tree: &mut Tree, parent_id: usize, value: &Value, key: &str) {
    match value {
        Value::Mapping(map) => {
            // Create a node for this mapping (object)
            let mut node = TreeNode::new(key, "mapping");

            // Add attribute for mapping size
            node.add_attribute("size", format!("{} fields", map.len()));

            let node_id = tree.add_node(node);
            tree.get_node_mut(parent_id).unwrap().add_child(node_id);

            // Recursively add children
            for (child_key, child_value) in map {
                // Convert key to string
                let key_str = match child_key {
                    Value::String(s) => s.clone(),
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => b.to_string(),
                    Value::Null => "null".to_string(),
                    _ => format!("{:?}", child_key),
                };
                convert_value(tree, node_id, child_value, &key_str);
            }
        }
        Value::Sequence(arr) => {
            // Create a node for this sequence (array)
            let mut node = TreeNode::new(key, "sequence");
            node.add_attribute("size", format!("{} items", arr.len()));

            let node_id = tree.add_node(node);
            tree.get_node_mut(parent_id).unwrap().add_child(node_id);

            // Recursively add children with indices
            for (index, item) in arr.iter().enumerate() {
                convert_value(tree, node_id, item, &format!("[{}]", index));
            }
        }
        Value::String(s) => {
            let mut node = TreeNode::new(key, TreeNode::ATTRIBUTE_TYPE);
            node.add_attribute("value", s.clone());
            let node_id = tree.add_node(node);
            tree.get_node_mut(parent_id).unwrap().add_child(node_id);
        }
        Value::Number(n) => {
            let mut node = TreeNode::new(key, TreeNode::ATTRIBUTE_TYPE);
            node.add_attribute("value", n.to_string());
            let node_id = tree.add_node(node);
            tree.get_node_mut(parent_id).unwrap().add_child(node_id);
        }
        Value::Bool(b) => {
            let mut node = TreeNode::new(key, TreeNode::ATTRIBUTE_TYPE);
            node.add_attribute("value", b.to_string());
            let node_id = tree.add_node(node);
            tree.get_node_mut(parent_id).unwrap().add_child(node_id);
        }
        Value::Null => {
            let mut node = TreeNode::new(key, TreeNode::ATTRIBUTE_TYPE);
            node.add_attribute("value", "null");
            let node_id = tree.add_node(node);
            tree.get_node_mut(parent_id).unwrap().add_child(node_id);
        }
        Value::Tagged(tagged) => {
            // Handle tagged values (e.g., !tag value)
            // For now, just process the inner value with a note about the tag
            let tag = &tagged.tag;
            let inner_value = &tagged.value;

            // Create a tagged node
            let mut node = TreeNode::new(key, "tagged");
            node.add_attribute("tag", tag.to_string());
            let node_id = tree.add_node(node);
            tree.get_node_mut(parent_id).unwrap().add_child(node_id);

            // Add the inner value as a child
            convert_value(tree, node_id, inner_value, "value");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_yaml() {
        let yaml = r#"
name: test
count: 42
"#;
        let parser = YamlParser;
        let tree = parser.parse(yaml).unwrap();

        assert!(tree.node_count() > 0);
        let root = tree.get_node(tree.root_id()).unwrap();
        assert_eq!(root.label, "root");
    }

    #[test]
    fn test_parse_nested_yaml() {
        let yaml = r#"
user:
  name: Alice
  age: 30
"#;
        let parser = YamlParser;
        let tree = parser.parse(yaml).unwrap();

        assert!(tree.node_count() > 3);
    }

    #[test]
    fn test_parse_yaml_array() {
        let yaml = r#"
items:
  - apple
  - banana
  - cherry
"#;
        let parser = YamlParser;
        let tree = parser.parse(yaml).unwrap();

        assert!(tree.node_count() > 4);
    }

    #[test]
    fn test_can_parse_yaml_extensions() {
        let parser = YamlParser;
        assert!(parser.can_parse(Path::new("test.yaml")));
        assert!(parser.can_parse(Path::new("test.yml")));
        assert!(parser.can_parse(Path::new("test.YAML")));
        assert!(parser.can_parse(Path::new("test.YML")));
        assert!(!parser.can_parse(Path::new("test.json")));
        assert!(!parser.can_parse(Path::new("test.xml")));
    }

    #[test]
    fn test_parse_yaml_types() {
        let yaml = r#"
string_value: "hello"
number_value: 123
float_value: 45.67
bool_true: true
bool_false: false
null_value: null
"#;
        let parser = YamlParser;
        let tree = parser.parse(yaml).unwrap();

        // Should have root + 6 attribute nodes
        assert!(tree.node_count() >= 7);
    }

    #[test]
    fn test_parse_complex_yaml() {
        let yaml = r#"
database:
  host: localhost
  port: 5432
  credentials:
    username: admin
    password: secret
servers:
  - name: web1
    ip: 192.168.1.10
  - name: web2
    ip: 192.168.1.11
"#;
        let parser = YamlParser;
        let tree = parser.parse(yaml).unwrap();

        // Complex structure should create many nodes
        // Root + root mapping + database mapping + host + port + credentials mapping + username + password
        // + servers sequence + 2 server mappings (each with name + ip)
        // = 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 2 + 4 = 15 nodes
        assert!(tree.node_count() >= 14);
    }
}

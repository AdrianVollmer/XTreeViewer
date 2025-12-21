use super::Parser;
use crate::error::{Result, XtvError};
use crate::tree::{Tree, TreeNode};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::path::Path;

pub struct XmlParser;

impl Parser for XmlParser {
    fn parse(&self, content: &str) -> Result<Tree> {
        let mut reader = Reader::from_str(content);
        reader.trim_text(true);

        let mut tree = Tree::new(TreeNode::new("root", "root"));
        let root_id = tree.root_id();

        // Stack to track parent nodes
        let mut parent_stack: Vec<usize> = vec![root_id];
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    let mut node = TreeNode::new(name, "element");

                    // Add XML attributes
                    for attr in e.attributes() {
                        let attr = attr.map_err(|e| XtvError::XmlParse(e.to_string()))?;
                        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                        let value = String::from_utf8_lossy(&attr.value).to_string();
                        node.add_attribute(key, value);
                    }

                    // Clone attributes before adding node to tree
                    let attributes = node.attributes.clone();
                    let node_id = tree.add_node(node);

                    // Create virtual attributes node if there are attributes
                    if let Some(virtual_id) = create_virtual_attributes_node(&mut tree, &attributes)
                    {
                        tree.get_node_mut(node_id)
                            .unwrap()
                            .children
                            .insert(0, virtual_id);
                    }

                    // Add as child to current parent
                    if let Some(&parent_id) = parent_stack.last() {
                        tree.get_node_mut(parent_id).unwrap().add_child(node_id);
                    }

                    // Push this node as the new parent
                    parent_stack.push(node_id);
                }
                Ok(Event::End(_)) => {
                    // Pop the current element from stack
                    parent_stack.pop();
                }
                Ok(Event::Text(e)) => {
                    let text = e
                        .unescape()
                        .map_err(|e| XtvError::XmlParse(e.to_string()))?
                        .trim()
                        .to_string();

                    // Only add non-empty text nodes
                    if !text.is_empty() {
                        let mut text_node = TreeNode::new("text", "text");
                        text_node.add_attribute("content", text);
                        let text_id = tree.add_node(text_node);

                        if let Some(&parent_id) = parent_stack.last() {
                            tree.get_node_mut(parent_id).unwrap().add_child(text_id);
                        }
                    }
                }
                Ok(Event::Empty(e)) => {
                    // Self-closing tag
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    let mut node = TreeNode::new(name, "element");

                    // Add XML attributes
                    for attr in e.attributes() {
                        let attr = attr.map_err(|e| XtvError::XmlParse(e.to_string()))?;
                        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                        let value = String::from_utf8_lossy(&attr.value).to_string();
                        node.add_attribute(key, value);
                    }

                    // Clone attributes before adding node to tree
                    let attributes = node.attributes.clone();
                    let node_id = tree.add_node(node);

                    // Create virtual attributes node if there are attributes
                    if let Some(virtual_id) = create_virtual_attributes_node(&mut tree, &attributes)
                    {
                        tree.get_node_mut(node_id)
                            .unwrap()
                            .children
                            .insert(0, virtual_id);
                    }

                    // Add as child to current parent
                    if let Some(&parent_id) = parent_stack.last() {
                        tree.get_node_mut(parent_id).unwrap().add_child(node_id);
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(XtvError::XmlParse(e.to_string())),
                _ => {} // Ignore other events
            }

            buf.clear();
        }

        Ok(tree)
    }

    fn can_parse(&self, file_path: &Path) -> bool {
        file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|s| s.eq_ignore_ascii_case("xml"))
            .unwrap_or(false)
    }
}

/// Creates a virtual "@attributes" node containing individual attribute nodes
/// Returns None if the attributes vector is empty
fn create_virtual_attributes_node(
    tree: &mut Tree,
    attributes: &[crate::tree::node::Attribute],
) -> Option<usize> {
    if attributes.is_empty() {
        return None;
    }

    // Create the virtual container node
    let mut virtual_node = TreeNode::new("@attributes", TreeNode::VIRTUAL_ATTRIBUTES_TYPE);

    // Create individual attribute nodes as children
    for attr in attributes {
        let mut attr_node = TreeNode::new(&attr.key, TreeNode::ATTRIBUTE_TYPE);
        attr_node.add_attribute("value", &attr.value);
        let attr_id = tree.add_node(attr_node);
        virtual_node.add_child(attr_id);
    }

    Some(tree.add_node(virtual_node))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_xml() {
        let xml = r#"<root><item>test</item></root>"#;
        let parser = XmlParser;
        let tree = parser.parse(xml).unwrap();

        assert!(tree.node_count() > 0);
    }

    #[test]
    fn test_parse_xml_with_attributes() {
        let xml = r#"<root id="1"><item name="test">value</item></root>"#;
        let parser = XmlParser;
        let tree = parser.parse(xml).unwrap();

        assert!(tree.node_count() > 2);
    }

    #[test]
    fn test_virtual_attributes_node_created() {
        let xml = r#"<root id="1" name="test"><child>value</child></root>"#;
        let parser = XmlParser;
        let tree = parser.parse(xml).unwrap();

        // Root element should have @attributes as first child
        let root = tree.get_node(0).unwrap();
        let root_element = tree.get_node(root.children[0]).unwrap();

        // First child should be virtual attributes node
        let first_child_id = root_element.children[0];
        let first_child = tree.get_node(first_child_id).unwrap();
        assert_eq!(first_child.node_type, "@attributes");

        // Virtual node should have 2 attribute children
        assert_eq!(first_child.children.len(), 2);
    }

    #[test]
    fn test_no_virtual_node_without_attributes() {
        let xml = r#"<root><child>value</child></root>"#;
        let parser = XmlParser;
        let tree = parser.parse(xml).unwrap();

        let root = tree.get_node(0).unwrap();
        let root_element = tree.get_node(root.children[0]).unwrap();

        // First child should be the <child> element, not @attributes
        let first_child = tree.get_node(root_element.children[0]).unwrap();
        assert_ne!(first_child.node_type, "@attributes");
    }

    #[test]
    fn test_individual_attribute_nodes() {
        let xml = r#"<item id="123" enabled="true">content</item>"#;
        let parser = XmlParser;
        let tree = parser.parse(xml).unwrap();

        let root = tree.get_node(0).unwrap();
        let item = tree.get_node(root.children[0]).unwrap();
        let virtual_node = tree.get_node(item.children[0]).unwrap();

        // Check first attribute node
        let attr1 = tree.get_node(virtual_node.children[0]).unwrap();
        assert_eq!(attr1.node_type, "attribute");
        assert_eq!(attr1.label, "id");
        assert_eq!(attr1.attributes[0].value, "123");

        // Check second attribute node
        let attr2 = tree.get_node(virtual_node.children[1]).unwrap();
        assert_eq!(attr2.label, "enabled");
        assert_eq!(attr2.attributes[0].value, "true");
    }
}

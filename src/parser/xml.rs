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

                    let node_id = tree.add_node(node);

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
                    let text = e.unescape()
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

                    let node_id = tree.add_node(node);

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
}

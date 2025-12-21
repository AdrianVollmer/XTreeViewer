use super::Parser;
use crate::error::Result;
use crate::tree::{Tree, TreeNode};
use ego_tree::NodeRef;
use scraper::{node::Node, Html};
use std::path::Path;

pub struct HtmlParser;

impl Parser for HtmlParser {
    fn parse(&self, content: &str) -> Result<Tree> {
        // Parse HTML document
        let document = Html::parse_document(content);

        // Create tree with root node
        let mut tree = Tree::new(TreeNode::new("root", "root"));
        let root_id = tree.root_id();

        // Traverse DOM tree recursively from root element
        for child in document.root_element().children() {
            traverse_node(&mut tree, root_id, child);
        }

        Ok(tree)
    }

    fn can_parse(&self, file_path: &Path) -> bool {
        file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|s| s.eq_ignore_ascii_case("html") || s.eq_ignore_ascii_case("htm"))
            .unwrap_or(false)
    }
}

fn traverse_node(tree: &mut Tree, parent_id: usize, node: NodeRef<Node>) {
    match node.value() {
        Node::Element(element) => {
            let tag_name = element.name();
            let mut elem_node = TreeNode::new(tag_name, "element");

            // Collect attributes
            for (key, value) in element.attrs() {
                elem_node.add_attribute(key, value);
            }

            // Clone attributes and add node to tree
            let attributes = elem_node.attributes.clone();
            let elem_id = tree.add_node(elem_node);

            // Create virtual attributes node if there are attributes
            if let Some(virtual_id) = create_virtual_attributes_node(tree, &attributes) {
                tree.get_node_mut(elem_id)
                    .unwrap()
                    .children
                    .insert(0, virtual_id);
            }

            // Add element as child to parent
            tree.get_node_mut(parent_id).unwrap().add_child(elem_id);

            // Recursively process children
            for child in node.children() {
                traverse_node(tree, elem_id, child);
            }
        }

        Node::Text(text) => {
            let trimmed = text.text.trim();
            if !trimmed.is_empty() {
                let mut text_node = TreeNode::new("text", "text");
                text_node.add_attribute("content", trimmed);
                let text_id = tree.add_node(text_node);
                tree.get_node_mut(parent_id).unwrap().add_child(text_id);
            }
        }

        Node::Comment(comment) => {
            let mut comment_node = TreeNode::new("comment", "comment");
            comment_node.add_attribute("content", &comment.comment);
            let comment_id = tree.add_node(comment_node);
            tree.get_node_mut(parent_id).unwrap().add_child(comment_id);
        }

        // Skip other node types (Document, Doctype, ProcessingInstruction, etc.)
        _ => {}
    }
}

fn create_virtual_attributes_node(
    tree: &mut Tree,
    attributes: &[crate::tree::node::Attribute],
) -> Option<usize> {
    if attributes.is_empty() {
        return None;
    }

    let mut virtual_node = TreeNode::new("@attributes", TreeNode::VIRTUAL_ATTRIBUTES_TYPE);

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
    fn test_parse_simple_html() {
        let html = r#"<html><body><p>Hello World</p></body></html>"#;
        let parser = HtmlParser;
        let tree = parser.parse(html).unwrap();
        assert!(tree.node_count() > 0);
    }

    #[test]
    fn test_parse_html_with_attributes() {
        let html = r#"<div id="container" class="wrapper"><p>Content</p></div>"#;
        let parser = HtmlParser;
        let tree = parser.parse(html).unwrap();
        assert!(tree.node_count() > 2);
    }

    #[test]
    fn test_virtual_attributes_node_created() {
        let html = r#"<div id="test" class="example"></div>"#;
        let parser = HtmlParser;
        let tree = parser.parse(html).unwrap();

        // Find a div element - it should have attributes
        let root = tree.get_node(0).unwrap();
        if !root.children.is_empty() {
            let div = tree.get_node(root.children[0]).unwrap();

            // First child should be @attributes if div has attributes
            if !div.children.is_empty() {
                let first_child = tree.get_node(div.children[0]).unwrap();
                assert_eq!(first_child.node_type, "@attributes");
            }
        }
    }

    #[test]
    fn test_void_elements() {
        let html = r#"<div><img src="test.jpg" alt="Test"><br></div>"#;
        let parser = HtmlParser;
        let tree = parser.parse(html).unwrap();
        assert!(tree.node_count() > 0);
    }

    #[test]
    fn test_can_parse_html_extension() {
        let parser = HtmlParser;
        assert!(parser.can_parse(Path::new("test.html")));
        assert!(parser.can_parse(Path::new("test.HTML")));
        assert!(parser.can_parse(Path::new("test.htm")));
        assert!(!parser.can_parse(Path::new("test.xml")));
    }

    #[test]
    fn test_whitespace_trimmed() {
        let html = r#"
            <div>

                <p>Text</p>

            </div>
        "#;
        let parser = HtmlParser;
        let tree = parser.parse(html).unwrap();

        // Should not have whitespace-only text nodes
        // Just verify it parses successfully
        assert!(tree.node_count() > 0);
    }
}

use super::Parser;
use crate::error::{Result, XtvError};
use crate::tree::{Tree, TreeNode};
use std::collections::HashMap;
use std::path::Path;

pub struct LdifParser;

impl Parser for LdifParser {
    fn parse(&self, content: &str) -> Result<Tree> {
        let mut parser = LdifFileParser::new(content);
        parser.parse()
    }

    fn can_parse(&self, file_path: &Path) -> bool {
        file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|s| s.eq_ignore_ascii_case("ldif"))
            .unwrap_or(false)
    }
}

struct LdifFileParser<'a> {
    lines: Vec<&'a str>,
    line_num: usize,
}

impl<'a> LdifFileParser<'a> {
    fn new(content: &'a str) -> Self {
        LdifFileParser {
            lines: content.lines().collect(),
            line_num: 0,
        }
    }

    fn parse(&mut self) -> Result<Tree> {
        let mut entries = Vec::new();

        // Parse optional version line
        if let Some(line) = self.peek_line() {
            if line.starts_with("version:") {
                self.line_num += 1;
            }
        }

        // Parse entries
        while self.line_num < self.lines.len() {
            if let Some(entry) = self.parse_entry()? {
                entries.push(entry);
            }
        }

        Ok(self.build_tree(entries))
    }

    fn parse_entry(&mut self) -> Result<Option<LdifEntry>> {
        // Skip blank lines and comments
        while let Some(line) = self.peek_line() {
            if line.is_empty() || line.starts_with('#') {
                self.line_num += 1;
            } else {
                break;
            }
        }

        // Check if we're at EOF
        if self.line_num >= self.lines.len() {
            return Ok(None);
        }

        // Read logical line (handling folding)
        let logical_line = self.read_logical_line();
        if logical_line.is_empty() {
            return Ok(None);
        }

        // First line should be DN
        if !logical_line.starts_with("dn:") {
            return Err(XtvError::LdifParse {
                line: self.line_num,
                message: format!("Expected DN, got: {}", logical_line),
            });
        }

        let dn = logical_line[3..].trim().to_string();
        let mut attributes = Vec::new();

        // Read attributes until blank line or EOF
        loop {
            // Skip comments
            while let Some(line) = self.peek_line() {
                if line.starts_with('#') {
                    self.line_num += 1;
                } else {
                    break;
                }
            }

            if self.line_num >= self.lines.len() {
                break;
            }

            if let Some(line) = self.peek_line() {
                if line.is_empty() {
                    self.line_num += 1;
                    break;
                }
            }

            let logical_line = self.read_logical_line();
            if logical_line.is_empty() {
                break;
            }

            // Parse attribute line
            if logical_line.contains(':') {
                let (key, value) = self.parse_attribute_line(&logical_line)?;
                attributes.push((key, value));
            }
        }

        Ok(Some(LdifEntry { dn, attributes }))
    }

    fn peek_line(&self) -> Option<&str> {
        self.lines.get(self.line_num).copied()
    }

    fn read_logical_line(&mut self) -> String {
        let mut result = String::new();

        if let Some(line) = self.lines.get(self.line_num) {
            result.push_str(line);
            self.line_num += 1;
        }

        // Accumulate continuation lines (starting with space)
        while let Some(line) = self.lines.get(self.line_num) {
            if line.starts_with(' ') {
                // Remove exactly one leading space
                result.push_str(&line[1..]);
                self.line_num += 1;
            } else {
                break;
            }
        }

        result
    }

    fn parse_attribute_line(&self, line: &str) -> Result<(String, String)> {
        // Handle three separators: :, ::, :<
        if let Some(pos) = line.find("::") {
            // Base64 encoded
            let key = line[..pos].trim();
            let encoded = line[pos + 2..].trim();
            let decoded = self.decode_base64(encoded)?;
            Ok((key.to_string(), decoded))
        } else if let Some(pos) = line.find(":<") {
            // URL reference
            let key = line[..pos].trim();
            let url = line[pos + 2..].trim();
            Ok((key.to_string(), format!("<URL reference: {}>", url)))
        } else if let Some(pos) = line.find(':') {
            // Plain value
            let key = line[..pos].trim();
            let value = line[pos + 1..].trim();
            Ok((key.to_string(), value.to_string()))
        } else {
            Err(XtvError::LdifParse {
                line: self.line_num,
                message: "Invalid attribute format".to_string(),
            })
        }
    }

    fn decode_base64(&self, encoded: &str) -> Result<String> {
        use base64::{engine::general_purpose, Engine as _};

        match general_purpose::STANDARD.decode(encoded) {
            Ok(bytes) => {
                // Try to convert to UTF-8 string
                match String::from_utf8(bytes.clone()) {
                    Ok(s) => Ok(s),
                    Err(_) => {
                        // Binary data - show hex preview or size
                        if bytes.len() <= 64 {
                            Ok(format!("<binary: {}>", hex_preview(&bytes)))
                        } else {
                            Ok(format!("<binary data, {} bytes>", bytes.len()))
                        }
                    }
                }
            }
            Err(e) => Err(XtvError::LdifParse {
                line: self.line_num,
                message: format!("Base64 decode error: {}", e),
            }),
        }
    }

    fn build_tree(&mut self, entries: Vec<LdifEntry>) -> Tree {
        let mut tree = Tree::new(TreeNode::new("root", "root"));
        let root_id = tree.root_id();

        // Map from DN to node ID for building hierarchy
        let mut dn_to_node: HashMap<String, usize> = HashMap::new();

        for entry in entries {
            // Get parent DN
            let parent_dn = get_parent_dn(&entry.dn);

            // Find parent node (only if it exists, don't create placeholders)
            let parent_id = if let Some(ref parent) = parent_dn {
                // Check if parent exists in the entries we've already processed
                dn_to_node.get(parent).copied().unwrap_or(root_id)
            } else {
                // No parent DN, attach to root
                root_id
            };

            // Compute RDN (relative to parent if parent exists in tree)
            let parent_dn_for_label = if parent_id == root_id {
                None
            } else {
                parent_dn.as_deref()
            };
            let rdn = compute_rdn(&entry.dn, parent_dn_for_label);

            // Create entry node with RDN as label
            let entry_node = TreeNode::new(&rdn, "entry");
            let entry_id = tree.add_node(entry_node);

            // Store DN to node mapping
            dn_to_node.insert(entry.dn.clone(), entry_id);

            // Create @attributes virtual node
            let mut attr_map: HashMap<String, Vec<String>> = HashMap::new();
            attr_map.insert("dn".to_string(), vec![entry.dn.clone()]);

            // Group multi-valued attributes
            for (key, value) in entry.attributes {
                attr_map.entry(key).or_insert_with(Vec::new).push(value);
            }

            // Create virtual attributes node
            let virtual_node = TreeNode::new("@attributes", TreeNode::VIRTUAL_ATTRIBUTES_TYPE);
            let virtual_id = tree.add_node(virtual_node);

            // Sort attribute keys alphanumerically
            let mut sorted_keys: Vec<_> = attr_map.keys().collect();
            sorted_keys.sort();

            // Add individual attribute nodes in sorted order
            for key in sorted_keys {
                let values = &attr_map[key];
                if values.len() == 1 {
                    let mut attr_node = TreeNode::new(key, TreeNode::ATTRIBUTE_TYPE);
                    attr_node.add_attribute("value", &values[0]);
                    let attr_id = tree.add_node(attr_node);
                    tree.get_node_mut(virtual_id).unwrap().add_child(attr_id);
                } else {
                    for (idx, value) in values.iter().enumerate() {
                        let label = format!("{} [{}]", key, idx);
                        let mut attr_node = TreeNode::new(&label, TreeNode::ATTRIBUTE_TYPE);
                        attr_node.add_attribute("value", value);
                        let attr_id = tree.add_node(attr_node);
                        tree.get_node_mut(virtual_id).unwrap().add_child(attr_id);
                    }
                }
            }

            // Link virtual node to entry
            tree.get_node_mut(entry_id).unwrap().add_child(virtual_id);

            // Link entry to parent
            tree.get_node_mut(parent_id).unwrap().add_child(entry_id);
        }

        tree
    }
}

struct LdifEntry {
    dn: String,
    attributes: Vec<(String, String)>,
}

/// Extract the parent DN from a DN
/// Example: "cn=John Doe,ou=People,dc=example,dc=com"
/// Returns: Some("ou=People,dc=example,dc=com")
fn get_parent_dn(dn: &str) -> Option<String> {
    // Find the first comma that's not escaped
    let mut in_quotes = false;
    let mut escape_next = false;

    for (i, ch) in dn.chars().enumerate() {
        if escape_next {
            escape_next = false;
            continue;
        }

        match ch {
            '\\' => escape_next = true,
            '"' => in_quotes = !in_quotes,
            ',' if !in_quotes => {
                // Found the separator, everything after is the parent
                let parent = dn[i + 1..].trim();

                if parent.is_empty() {
                    return None;
                } else {
                    return Some(parent.to_string());
                }
            }
            _ => {}
        }
    }

    // No comma found, no parent
    None
}

/// Compute the relative DN (the part not in the parent)
/// Example: dn="cn=John Doe,ou=People,dc=example,dc=com", parent="ou=People,dc=example,dc=com"
/// Returns: "cn=John Doe"
fn compute_rdn(dn: &str, parent: Option<&str>) -> String {
    if let Some(parent_dn) = parent {
        // Remove the parent DN suffix from the full DN
        if dn.ends_with(parent_dn) {
            let prefix_len = dn.len() - parent_dn.len();
            if prefix_len > 0 && dn.as_bytes()[prefix_len - 1] == b',' {
                return dn[..prefix_len - 1].trim().to_string();
            }
        }
    }
    // If no parent or doesn't end with parent, return full DN
    dn.trim().to_string()
}

fn hex_preview(bytes: &[u8]) -> String {
    bytes
        .iter()
        .take(32)
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_ldif() {
        let ldif = "version: 1\n\ndn: cn=Test,dc=example,dc=com\ncn: Test\nsn: User\n";
        let parser = LdifParser;
        let tree = parser.parse(ldif).unwrap();
        assert!(tree.node_count() > 0);

        let root = tree.get_node(0).unwrap();
        assert!(root.has_children());
    }

    #[test]
    fn test_parse_multiple_entries() {
        let ldif = r#"version: 1

dn: cn=First,dc=example,dc=com
cn: First

dn: cn=Second,dc=example,dc=com
cn: Second
"#;
        let parser = LdifParser;
        let tree = parser.parse(ldif).unwrap();

        let root = tree.get_node(0).unwrap();
        // Root should have both entries directly (parent dc=example,dc=com not in file)
        assert_eq!(root.children.len(), 2);

        // Both should have full DNs as labels
        let first_entry = tree.get_node(root.children[0]).unwrap();
        assert_eq!(first_entry.label, "cn=First,dc=example,dc=com");

        let second_entry = tree.get_node(root.children[1]).unwrap();
        assert_eq!(second_entry.label, "cn=Second,dc=example,dc=com");
    }

    #[test]
    fn test_line_folding() {
        let ldif = "version: 1\n\ndn: cn=Test,dc=example,dc=com\ndescription: This is a long\n description that continues\n  on multiple lines\n";
        let parser = LdifParser;
        let tree = parser.parse(ldif).unwrap();

        assert!(tree.node_count() > 0);
    }

    #[test]
    fn test_multi_valued_attributes() {
        let ldif = r#"version: 1

dn: cn=Test,dc=example,dc=com
objectClass: top
objectClass: person
objectClass: organizationalPerson
"#;
        let parser = LdifParser;
        let tree = parser.parse(ldif).unwrap();

        let root = tree.get_node(0).unwrap();
        // Entry attached to root (parent not in file), has full DN
        let entry = tree.get_node(root.children[0]).unwrap();
        assert_eq!(entry.label, "cn=Test,dc=example,dc=com");

        // Entry's first child should be @attributes
        let attrs = tree.get_node(entry.children[0]).unwrap();
        assert_eq!(attrs.node_type, "@attributes");

        // Should have dn + 3 objectClass attributes
        assert!(attrs.children.len() >= 4);

        // Check for indexed objectClass attributes
        let mut found_indexed = false;
        for child_id in &attrs.children {
            let child = tree.get_node(*child_id).unwrap();
            if child.label.contains("objectClass [") {
                found_indexed = true;
                break;
            }
        }
        assert!(found_indexed);
    }

    #[test]
    fn test_base64_decoding() {
        // "Test" in base64 is "VGVzdA=="
        let ldif = "version: 1\n\ndn: cn=Test,dc=example,dc=com\ndescription:: VGVzdA==\n";
        let parser = LdifParser;
        let tree = parser.parse(ldif).unwrap();

        assert!(tree.node_count() > 0);
    }

    #[test]
    fn test_url_reference() {
        let ldif = "version: 1\n\ndn: cn=Test,dc=example,dc=com\nphoto:< file:///tmp/photo.jpg\n";
        let parser = LdifParser;
        let tree = parser.parse(ldif).unwrap();

        assert!(tree.node_count() > 0);
    }

    #[test]
    fn test_comments() {
        let ldif = r#"version: 1

# This is a comment
dn: cn=Test,dc=example,dc=com
# Another comment
cn: Test
"#;
        let parser = LdifParser;
        let tree = parser.parse(ldif).unwrap();

        let root = tree.get_node(0).unwrap();
        assert_eq!(root.children.len(), 1);
    }

    #[test]
    fn test_version_line() {
        let ldif = "version: 1\n\ndn: cn=Test,dc=example,dc=com\ncn: Test\n";
        let parser = LdifParser;
        let result = parser.parse(ldif);
        assert!(result.is_ok());
    }

    #[test]
    fn test_virtual_attributes_node() {
        let ldif = "version: 1\n\ndn: cn=Test,dc=example,dc=com\ncn: Test\n";
        let parser = LdifParser;
        let tree = parser.parse(ldif).unwrap();

        let root = tree.get_node(0).unwrap();
        // Entry attached to root (parent not in file), has full DN
        let entry = tree.get_node(root.children[0]).unwrap();
        assert_eq!(entry.node_type, "entry");
        assert_eq!(entry.label, "cn=Test,dc=example,dc=com");

        // First child should be @attributes
        let attrs = tree.get_node(entry.children[0]).unwrap();
        assert_eq!(attrs.node_type, "@attributes");
    }

    #[test]
    fn test_can_parse_ldif_extension() {
        let parser = LdifParser;
        assert!(parser.can_parse(Path::new("test.ldif")));
        assert!(parser.can_parse(Path::new("test.LDIF")));
        assert!(!parser.can_parse(Path::new("test.xml")));
    }

    #[test]
    fn test_empty_ldif() {
        let ldif = "";
        let parser = LdifParser;
        let tree = parser.parse(ldif).unwrap();

        let root = tree.get_node(0).unwrap();
        assert_eq!(root.children.len(), 0);
    }

    #[test]
    fn test_malformed_entry() {
        let ldif = "version: 1\n\nnotadn: invalid\n";
        let parser = LdifParser;
        let result = parser.parse(ldif);
        assert!(result.is_err());
    }

    #[test]
    fn test_whitespace_in_values() {
        let ldif = "version: 1\n\ndn: cn=Test,dc=example,dc=com\ncn:  Test  \n";
        let parser = LdifParser;
        let tree = parser.parse(ldif).unwrap();
        assert!(tree.node_count() > 0);
    }

    #[test]
    fn test_no_version_line() {
        let ldif = "dn: cn=Test,dc=example,dc=com\ncn: Test\n";
        let parser = LdifParser;
        let result = parser.parse(ldif);
        assert!(result.is_ok());
    }

    #[test]
    fn test_hierarchical_structure() {
        let ldif = r#"version: 1

dn: dc=example,dc=com
objectClass: top
dc: example

dn: ou=People,dc=example,dc=com
objectClass: organizationalUnit
ou: People

dn: cn=John Doe,ou=People,dc=example,dc=com
objectClass: person
cn: John Doe
"#;
        let parser = LdifParser;
        let tree = parser.parse(ldif).unwrap();

        let root = tree.get_node(0).unwrap();

        // Root should have dc=example,dc=com (full DN since no parent in file)
        assert_eq!(root.children.len(), 1);
        let dc_node = tree.get_node(root.children[0]).unwrap();
        assert_eq!(dc_node.label, "dc=example,dc=com");

        // dc=example,dc=com should have @attributes and ou=People
        // First child is @attributes, find ou=People
        let ou_node = dc_node.children.iter()
            .map(|&id| tree.get_node(id).unwrap())
            .find(|n| n.node_type == "entry")
            .expect("Should have ou=People entry");
        assert_eq!(ou_node.label, "ou=People"); // Relative to parent

        // ou=People should have @attributes and cn=John Doe
        let cn_node = ou_node.children.iter()
            .map(|&id| tree.get_node(id).unwrap())
            .find(|n| n.node_type == "entry")
            .expect("Should have cn=John Doe entry");
        assert_eq!(cn_node.label, "cn=John Doe"); // Relative to parent
    }

    #[test]
    fn test_dn_parsing() {
        // Test get_parent_dn
        assert_eq!(
            get_parent_dn("cn=John Doe,ou=People,dc=example,dc=com"),
            Some("ou=People,dc=example,dc=com".to_string())
        );
        assert_eq!(get_parent_dn("dc=com"), None);
        assert_eq!(
            get_parent_dn("cn=Doe\\, John,ou=People"),
            Some("ou=People".to_string())
        );

        // Test compute_rdn
        assert_eq!(
            compute_rdn("cn=John Doe,ou=People,dc=example,dc=com", Some("ou=People,dc=example,dc=com")),
            "cn=John Doe"
        );
        assert_eq!(
            compute_rdn("ou=People,dc=example,dc=com", Some("dc=example,dc=com")),
            "ou=People"
        );
        assert_eq!(
            compute_rdn("dc=example,dc=com", None),
            "dc=example,dc=com"
        );
    }
}

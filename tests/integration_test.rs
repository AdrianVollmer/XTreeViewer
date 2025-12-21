use std::fs;
use std::path::PathBuf;
use xtv::parser;
use xtv::parser::Parser;

#[test]
fn test_parse_sample_json() {
    let path = PathBuf::from("examples/sample.json");
    let content = fs::read_to_string(&path).expect("Failed to read sample.json");

    let parser = parser::detect_parser(&path).expect("Failed to detect parser");
    let tree = parser.parse(&content).expect("Failed to parse JSON");

    // Verify tree structure
    assert!(tree.node_count() > 0);
    let root = tree.get_node(tree.root_id()).unwrap();
    assert_eq!(root.label, "root");
    assert!(root.has_children());
}

#[test]
fn test_parse_sample_xml() {
    let path = PathBuf::from("examples/sample.xml");
    let content = fs::read_to_string(&path).expect("Failed to read sample.xml");

    let parser = parser::detect_parser(&path).expect("Failed to detect parser");
    let tree = parser.parse(&content).expect("Failed to parse XML");

    // Verify tree structure
    assert!(tree.node_count() > 0);
    let root = tree.get_node(tree.root_id()).unwrap();
    assert_eq!(root.label, "root");
    assert!(root.has_children());
}

#[test]
fn test_parse_sample_ldif() {
    let path = PathBuf::from("examples/sample.ldif");
    let content = fs::read_to_string(&path).expect("Failed to read sample.ldif");

    let parser = parser::detect_parser(&path).expect("Failed to detect parser");
    let tree = parser.parse(&content).expect("Failed to parse LDIF");

    // Verify tree structure
    assert!(tree.node_count() > 0);
    let root = tree.get_node(tree.root_id()).unwrap();
    assert_eq!(root.label, "root");
    assert!(root.has_children());

    // Verify we have entries
    assert!(root.children.len() >= 4); // At least 4 entries in sample.ldif
}

#[test]
fn test_parse_sample_complex_ldif() {
    let path = PathBuf::from("examples/sample-complex.ldif");
    let content = fs::read_to_string(&path).expect("Failed to read sample-complex.ldif");

    let parser = parser::detect_parser(&path).expect("Failed to detect parser");
    let tree = parser.parse(&content).expect("Failed to parse LDIF");

    assert!(tree.node_count() > 0);
}

#[test]
fn test_ldif_entry_structure() {
    let ldif = "version: 1\n\ndn: cn=Test,dc=example,dc=com\ncn: Test\nsn: User\n";
    let parser = xtv::parser::ldif::LdifParser;
    let tree = parser.parse(ldif).unwrap();

    let root = tree.get_node(0).unwrap();
    assert!(root.has_children());

    // Get first entry
    let entry = tree.get_node(root.children[0]).unwrap();
    assert_eq!(entry.node_type, "entry");
    assert_eq!(entry.label, "cn=Test,dc=example,dc=com");

    // Entry should have @attributes child
    let attrs = tree.get_node(entry.children[0]).unwrap();
    assert_eq!(attrs.node_type, "@attributes");
    assert!(attrs.has_children());
}

#[test]
fn test_ldif_multi_valued_attributes() {
    let ldif = r#"version: 1

dn: cn=Test,dc=example,dc=com
objectClass: top
objectClass: person
mail: first@example.com
mail: second@example.com
"#;
    let parser = xtv::parser::ldif::LdifParser;
    let tree = parser.parse(ldif).unwrap();

    let root = tree.get_node(0).unwrap();
    let entry = tree.get_node(root.children[0]).unwrap();
    let attrs = tree.get_node(entry.children[0]).unwrap();

    // Should have multiple objectClass and mail attributes with indices
    let mut found_indexed = false;
    for child_id in &attrs.children {
        let child = tree.get_node(*child_id).unwrap();
        if child.label.contains("[0]") || child.label.contains("[1]") {
            found_indexed = true;
            break;
        }
    }
    assert!(found_indexed);
}

#[test]
fn test_unsupported_format() {
    let path = PathBuf::from("test.unsupported");
    let result = parser::detect_parser(&path);

    assert!(result.is_err());
}

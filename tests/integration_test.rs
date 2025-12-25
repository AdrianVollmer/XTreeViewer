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

    // Root should have dc=example,dc=com as the top-level entry
    assert!(root.children.len() >= 1);
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

    // Entry attached to root (parent not in file), has full DN
    let entry = tree.get_node(root.children[0]).unwrap();
    assert_eq!(entry.node_type, "entry");
    assert_eq!(entry.label, "cn=Test,dc=example,dc=com"); // Full DN since parent not in file

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
    // Entry attached to root (parent not in file), has full DN
    let entry = tree.get_node(root.children[0]).unwrap();
    assert_eq!(entry.label, "cn=Test,dc=example,dc=com");

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
fn test_parse_sample_yaml() {
    let path = PathBuf::from("examples/sample.yaml");
    let content = fs::read_to_string(&path).expect("Failed to read sample.yaml");

    let parser = parser::detect_parser(&path).expect("Failed to detect parser");
    let tree = parser.parse(&content).expect("Failed to parse YAML");

    // Verify tree structure
    assert!(tree.node_count() > 0);
    let root = tree.get_node(tree.root_id()).unwrap();
    assert_eq!(root.label, "root");
    assert!(root.has_children());
}

#[test]
fn test_yaml_nested_structure() {
    let yaml = r#"
database:
  host: localhost
  port: 5432
  credentials:
    username: admin
    password: secret
"#;
    let parser = xtv::parser::yaml::YamlParser;
    let tree = parser.parse(yaml).unwrap();

    let root = tree.get_node(0).unwrap();
    assert!(root.has_children());

    // Should have the root mapping node
    assert!(tree.node_count() > 5);
}

#[test]
fn test_yaml_arrays() {
    let yaml = r#"
items:
  - apple
  - banana
  - cherry
"#;
    let parser = xtv::parser::yaml::YamlParser;
    let tree = parser.parse(yaml).unwrap();

    // Should have root + root mapping + items sequence + 3 items
    // = 1 + 1 + 1 + 3 = 6 nodes
    assert!(tree.node_count() >= 5);
}

#[test]
fn test_parse_sample_jsonlines() {
    let path = PathBuf::from("examples/sample.jsonl");
    let content = fs::read_to_string(&path).expect("Failed to read sample.jsonl");

    let parser = parser::detect_parser(&path).expect("Failed to detect parser");
    let tree = parser.parse(&content).expect("Failed to parse JSON Lines");

    // Verify tree structure
    assert!(tree.node_count() > 0);
    let root = tree.get_node(tree.root_id()).unwrap();
    assert_eq!(root.label, "root");
    assert!(root.has_children());

    // Should have 7 children (one for each line in sample.jsonl)
    assert_eq!(root.children.len(), 7);
}

#[test]
fn test_jsonlines_line_numbering() {
    let jsonl = r#"{"id": 1}
{"id": 2}
{"id": 3}"#;
    let parser = xtv::parser::jsonlines::JsonLinesParser;
    let tree = parser.parse(jsonl).unwrap();

    let root = tree.get_node(0).unwrap();
    assert_eq!(root.children.len(), 3);

    // Check that lines are numbered starting from 1
    let first_line = tree.get_node(root.children[0]).unwrap();
    assert_eq!(first_line.label, "[1]");

    let second_line = tree.get_node(root.children[1]).unwrap();
    assert_eq!(second_line.label, "[2]");

    let third_line = tree.get_node(root.children[2]).unwrap();
    assert_eq!(third_line.label, "[3]");
}

#[test]
fn test_unsupported_format() {
    let path = PathBuf::from("test.unsupported");
    let result = parser::detect_parser(&path);

    assert!(result.is_err());
}
